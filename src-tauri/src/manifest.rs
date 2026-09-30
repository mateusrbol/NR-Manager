use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{AppError, Result};
use crate::logging::Logger;
use crate::models::{FileRecord, Manifest, ModifiedRecord};
use crate::settings::MANIFEST_FILE_NAME;
use crate::util;

/// Resultado de uma varredura da pasta.
#[derive(Default, Clone, Debug)]
pub struct Snapshot {
    pub files: Vec<FileRecord>,
    pub dirs: Vec<String>,
}

/// Mapeia arquivos originais salvos em backup.
#[derive(Default)]
pub struct BackupMap {
    /// path relativo -> caminho relativo do backup em backup_dir
    pub entries: HashMap<String, String>,
    /// arquivos grandes demais para backup
    pub unbacked: Vec<String>,
}

/// Snapshot recursivo (arquivo + tamanho + mtime + hash quando pequeno).
pub fn snapshot(dir: &Path, cap: u64, on_file: &dyn Fn(usize)) -> Result<Snapshot> {
    if !dir.exists() {
        return Err(AppError::NotFound(util::to_win_string(dir)));
    }
    let mut snap = Snapshot::default();
    let mut count = 0usize;

    for entry in WalkDir::new(dir).follow_links(false).into_iter().flatten() {
        let path = entry.path();
        if path == dir {
            continue;
        }
        let rel = match path.strip_prefix(dir) {
            Ok(r) => util::to_win_string(r),
            Err(_) => continue,
        };
        // Ignora o proprio manifesto e downloads em andamento.
        if path.file_name().map(|n| n == MANIFEST_FILE_NAME).unwrap_or(false)
            || rel.ends_with(".download")
        {
            continue;
        }

        if entry.file_type().is_dir() {
            snap.dirs.push(rel);
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let sha256 = if size <= cap {
            util::hash_file(path)
        } else {
            String::new()
        };

        snap.files.push(FileRecord {
            path: rel,
            size,
            mtime,
            sha256,
        });

        count += 1;
        on_file(count);
    }

    snap.files.sort_by(|a, b| a.path.cmp(&b.path));
    snap.dirs.sort();
    Ok(snap)
}

/// Copia para backup todos os arquivos originais pequenos (<= cap).
pub fn make_backup(
    before: &Snapshot,
    mod_dir: &Path,
    backup_dir: &Path,
    cap: u64,
    logger: &Logger,
) -> Result<BackupMap> {
    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(backup_dir);
    }
    std::fs::create_dir_all(backup_dir)?;

    let mut map = BackupMap::default();
    for rec in &before.files {
        if rec.size > cap {
            map.unbacked.push(rec.path.clone());
            continue;
        }
        let src = mod_dir.join(&rec.path);
        let dst = backup_dir.join(&rec.path);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::copy(&src, &dst) {
            Ok(_) => {
                map.entries.insert(rec.path.clone(), rec.path.clone());
            }
            Err(e) => {
                logger.warn(format!(
                    "Falha ao salvar backup de '{}': {e}",
                    rec.path
                ));
                map.unbacked.push(rec.path.clone());
            }
        }
    }
    logger.info(format!(
        "Backup criado: {} arquivo(s) em {} ({} nao salvos por tamanho).",
        map.entries.len(),
        backup_dir.display(),
        map.unbacked.len()
    ));
    Ok(map)
}

/// Monta o manifesto a partir da diferenca antes/depois.
#[allow(clippy::too_many_arguments)]
pub fn build_manifest(
    game_id: &str,
    game_name: &str,
    version: &str,
    setup_sha: Option<String>,
    mod_dir: &Path,
    backup_dir: &Path,
    cap: u64,
    before: &Snapshot,
    backup: &BackupMap,
) -> Result<Manifest> {
    let after = snapshot(mod_dir, cap, &|_| {})?;

    let before_map: HashMap<&str, &FileRecord> =
        before.files.iter().map(|f| (f.path.as_str(), f)).collect();
    let after_map: HashMap<&str, &FileRecord> =
        after.files.iter().map(|f| (f.path.as_str(), f)).collect();

    let mut created_files = Vec::new();
    for f in &after.files {
        if !before_map.contains_key(f.path.as_str()) {
            created_files.push(f.clone());
        }
    }

    let mut modified_files = Vec::new();
    let mut unbacked_changed = Vec::new();
    for f in &after.files {
        if let Some(b) = before_map.get(f.path.as_str()) {
            if changed(b, f) {
                let backup_path = backup.entries.get(&f.path).cloned();
                let backed_up = backup_path.is_some();
                if !backed_up && backup.unbacked.contains(&f.path) {
                    unbacked_changed.push(f.path.clone());
                }
                modified_files.push(ModifiedRecord {
                    path: f.path.clone(),
                    sha256: f.sha256.clone(),
                    size: f.size,
                    mtime: f.mtime,
                    backup_path,
                    backed_up,
                });
            }
        }
    }

    let mut deleted_files = Vec::new();
    for f in &before.files {
        if !after_map.contains_key(f.path.as_str()) {
            let backup_path = backup.entries.get(&f.path).cloned();
            let backed_up = backup_path.is_some();
            if !backed_up && backup.unbacked.contains(&f.path) {
                unbacked_changed.push(f.path.clone());
            }
            deleted_files.push(ModifiedRecord {
                path: f.path.clone(),
                sha256: f.sha256.clone(),
                size: f.size,
                mtime: f.mtime,
                backup_path,
                backed_up,
            });
        }
    }

    let before_dirs: std::collections::HashSet<&str> =
        before.dirs.iter().map(|s| s.as_str()).collect();
    let created_dirs: Vec<String> = after
        .dirs
        .iter()
        .filter(|d| !before_dirs.contains(d.as_str()))
        .cloned()
        .collect();

    Ok(Manifest {
        schema_version: 1,
        game_id: game_id.to_string(),
        game_name: game_name.to_string(),
        mod_version: version.to_string(),
        setup_sha256: setup_sha,
        installed_at: chrono::Utc::now().to_rfc3339(),
        mod_dir: util::to_win_string(mod_dir),
        backup_dir: util::to_win_string(backup_dir),
        created_files,
        modified_files,
        deleted_files,
        created_dirs,
        unbacked_files: unbacked_changed,
    })
}

fn changed(before: &FileRecord, after: &FileRecord) -> bool {
    if !before.sha256.is_empty() && !after.sha256.is_empty() {
        !before.sha256.eq_ignore_ascii_case(&after.sha256)
    } else {
        before.size != after.size || before.mtime != after.mtime
    }
}

/// Remove tudo que o manifesto registrou e restaura os originais.
pub fn uninstall(manifest: &Manifest, logger: &Logger) -> Result<()> {
    let mod_dir = PathBuf::from(&manifest.mod_dir);
    let backup_dir = PathBuf::from(&manifest.backup_dir);

    // 1) Remove arquivos criados pelo mod.
    for f in &manifest.created_files {
        let p = mod_dir.join(&f.path);
        if p.exists() {
            if let Err(e) = std::fs::remove_file(&p) {
                logger.warn(format!("Nao foi possivel remover '{}': {e}", f.path));
            }
        }
    }

    // 2) Restaura arquivos modificados.
    restore_all(&manifest.modified_files, &mod_dir, &backup_dir, logger);
    // 3) Restaura arquivos removidos pelo instalador.
    restore_all(&manifest.deleted_files, &mod_dir, &backup_dir, logger);

    // 4) Remove diretorios criados (mais profundos primeiro).
    let mut dirs = manifest.created_dirs.clone();
    dirs.sort_by_key(|d| std::cmp::Reverse(d.matches('\\').count()));
    for d in dirs {
        let p = mod_dir.join(&d);
        if p.exists() {
            let _ = std::fs::remove_dir(&p); // so remove se vazio
        }
    }

    // 5) Remove os manifestos.
    let _ = std::fs::remove_file(mod_dir.join(MANIFEST_FILE_NAME));
    Ok(())
}

fn restore_all(records: &[ModifiedRecord], mod_dir: &Path, backup_dir: &Path, logger: &Logger) {
    for r in records {
        let Some(rel) = &r.backup_path else {
            logger.warn(format!(
                "Sem backup para '{}': nao foi possivel restaurar o original.",
                r.path
            ));
            continue;
        };
        let src = backup_dir.join(rel);
        let dst = mod_dir.join(&r.path);
        if !src.exists() {
            logger.warn(format!("Backup ausente para '{}'.", r.path));
            continue;
        }
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::copy(&src, &dst) {
            logger.warn(format!("Falha ao restaurar '{}': {e}", r.path));
        }
    }
}
