pub mod epic;
pub mod gog;
pub mod steam;
pub mod xbox;

use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::error::Result;
use crate::logging::Logger;
use crate::models::{CompatEntry, Game, Manifest};
use crate::paths::Paths;
use crate::settings::MANIFEST_FILE_NAME;
use crate::util;

/// Ponto de entrada: detecta todos os jogos instalados nos lancadores suportados.
pub fn detect_all(logger: &Logger) -> Vec<Game> {
    let mut out: Vec<Game> = Vec::new();
    out.extend(steam::detect(logger));
    out.extend(epic::detect(logger));
    out.extend(gog::detect(logger));
    out.extend(xbox::detect(logger));

    // Remove duplicados por pasta de instalacao.
    let mut seen = std::collections::HashSet::new();
    out.retain(|g| seen.insert(util::norm_path(Path::new(&g.install_dir))));
    logger.info(format!("Deteccao concluida: {} jogos encontrados.", out.len()));
    out
}

/// Le a lista de compatibilidade do AppData.
pub fn load_compat(paths: &Paths) -> Vec<CompatEntry> {
    let text = std::fs::read_to_string(paths.compat_file()).unwrap_or_default();
    #[derive(serde::Deserialize)]
    struct Wrapper {
        #[serde(default)]
        entries: Vec<CompatEntry>,
    }
    serde_json::from_str::<Wrapper>(&text)
        .map(|w| w.entries)
        .unwrap_or_default()
}

/// Preenche API grafica, compatibilidade, anti-cheat, FSR, exe e pasta do mod.
pub fn enrich(game: &mut Game, compat: &[CompatEntry], logger: &Logger) {
    let entry = find_compat_entry(game, compat);

    if let Some(e) = entry {
        if !e.graphics_api.is_empty() && e.graphics_api != "unknown" {
            game.graphics_api = e.graphics_api.clone();
        }
        game.compat = e.status.clone();
        game.compat_note = e.notes.clone();
        if e.anti_cheat.is_some() {
            game.anti_cheat = e.anti_cheat.clone();
        }
    }

    // Localiza o executavel / pasta do mod.
    // Reavalia tambem quando o exe atual esta na raiz da instalacao: jogos UE
    // costumam ter um stub na raiz e o exe real em <Jogo>\Binaries\Win64\.
    let exe_at_root = game
        .exe_path
        .as_ref()
        .map(|p| {
            let exe = Path::new(p);
            !exe.exists()
                || util::path_eq(
                    exe.parent().unwrap_or(Path::new("")),
                    Path::new(&game.install_dir),
                )
        })
        .unwrap_or(true);
    if game.exe_path.is_none() || game.mod_dir.is_none() || exe_at_root {
        locate_executable(game, entry, logger);
    }

    // Heuristicas de FSR / DX12 na pasta do jogo.
    let scan_dir = game
        .mod_dir
        .clone()
        .or_else(|| Some(game.install_dir.clone()))
        .map(PathBuf::from);
    if let Some(dir) = scan_dir {
        let (fsr, dx12, vk) = detect_fsr(&dir, Path::new(&game.install_dir));
        game.fsr_detected = fsr;
        if game.graphics_api.is_empty() || game.graphics_api == "unknown" {
            if dx12 {
                game.graphics_api = "dx12".into();
            } else if vk {
                game.graphics_api = "vulkan".into();
            }
        }
        // Ajusta "untested" para "probable" quando ha evidencia de DX12 + FSR.
        if game.compat == "untested" || game.compat.is_empty() {
            if dx12 && fsr {
                game.compat = "probable".into();
                game.compat_note = Some(
                    "Detectado DX12 e FSR na pasta: provavelmente compativel (nao validado)."
                        .into(),
                );
            }
        }
        // Vulkan sem suporte por enquanto.
        if game.graphics_api == "vulkan" && game.compat != "compatible" {
            game.compat = "incompatible".into();
            game.compat_note = Some("Vulkan ainda nao e suportado pelo mod.".into());
        }
    }

    // Anti-cheat por arquivos conhecidos.
    if game.anti_cheat.is_none() {
        game.anti_cheat = detect_anti_cheat(Path::new(&game.install_dir));
    }

    if game.compat.is_empty() {
        game.compat = "untested".into();
    }

    // Aviso forte de anti-cheat no texto do card.
    if let Some(ac) = &game.anti_cheat {
        if game.compat_note.is_none() {
            game.compat_note = Some(format!(
                "Aviso: anti-cheat detectado ({ac}). DLL modificada pode causar bloqueio. Use so offline."
            ));
        }
    }
}

/// Retorna a entrada de compatibilidade correspondente ao jogo.
pub fn find_compat_entry<'a>(game: &Game, compat: &'a [CompatEntry]) -> Option<&'a CompatEntry> {
    // 1) appid do lancador
    if let (Some(app_id), source) = (&game.app_id, game.source.as_str()) {
        if let Some(e) = compat.iter().find(|e| {
            e.app_ids
                .get(source)
                .map(|v| v == app_id)
                .unwrap_or(false)
        }) {
            return Some(e);
        }
    }
    // 2) nome exato / alias normalizado
    let n = util::norm_name(&game.name);
    compat.iter().find(|e| {
        util::norm_name(&e.name) == n
            || e.aliases.iter().any(|a| util::norm_name(a) == n)
    })
}

fn locate_executable(game: &mut Game, entry: Option<&CompatEntry>, logger: &Logger) {
    let install = PathBuf::from(&game.install_dir);
    if !install.exists() {
        return;
    }

    // Caminho conhecido pela compatibilidade.
    if let Some(e) = entry {
        if !e.exe_name.is_empty() {
            let mut candidate = install.clone();
            if !e.exe_subdir.trim().is_empty() {
                candidate = candidate.join(e.exe_subdir.trim());
            }
            let exe = candidate.join(&e.exe_name);
            if exe.exists() {
                game.exe_path = Some(util::to_win_string(&exe));
                game.mod_dir = Some(util::to_win_string(exe.parent().unwrap()));
                return;
            }
        }
    }

    // Heuristica geral.
    match find_executable(&install, &game.name) {
        Some(exe) => {
            game.mod_dir = Some(util::to_win_string(exe.parent().unwrap()));
            game.exe_path = Some(util::to_win_string(&exe));
        }
        None => {
            logger.warn(format!(
                "Nao foi possivel localizar o executavel de '{}'. Corrija manualmente na tela do jogo.",
                game.name
            ));
        }
    }
}

/// Encontra o executavel principal do jogo de forma heuristica.
pub fn find_executable(install: &Path, game_name: &str) -> Option<PathBuf> {
    let score_re = Regex::new(r"(?i)^(launcher|setup|unins|vc_?redist|dxsetup|crash|report|helper|updater|eac|battleye|beservice|vgc|easyanticheat)").unwrap();
    let bad_dirs = [
        "redist", "_commonredist", "directx", "dotnet", "vcredist", "prereq", "support",
        "tools", "mods", "crashreporter", "easyanticheat", "battleye", "eac", "anticheat",
        "engine", "content", "movies", "saved", ".nrmanager",
    ];
    let name_norm = util::norm_name(game_name);
    let mut best: Option<(i64, PathBuf)> = None;

    for entry in WalkDir::new(install)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let p = util::norm_path(e.path());
                !bad_dirs.iter().any(|b| p.ends_with(&util::norm_path(Path::new(*b))))
            } else {
                true
            }
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_exe = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("exe"))
            .unwrap_or(false);
        if !is_exe {
            continue;
        }
        let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if score_re.is_match(&fname) {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let stem_norm = util::norm_name(&stem);
        let size_mb = entry.metadata().map(|m| m.len() / (1024 * 1024)).unwrap_or(0);

        let mut score: i64 = 0;
        if stem_norm == name_norm {
            score += 1000;
        } else if !name_norm.is_empty()
            && (stem_norm.contains(&name_norm) || name_norm.contains(&stem_norm))
        {
            score += 400;
        }
        // Prefere pastas tipicas de binarios.
        let dir_norm = util::norm_path(path.parent().unwrap_or(Path::new("")));
        if dir_norm.ends_with("bin\\x64")
            || dir_norm.ends_with("binaries\\win64")
            || dir_norm.ends_with("bin")
            || dir_norm.ends_with("x64")
            || dir_norm.contains("win64")
        {
            score += 150;
        }
        score += (size_mb.min(400)) as i64 / 2;

        match &best {
            Some((bs, _)) if *bs >= score => {}
            _ => best = Some((score, path.to_path_buf())),
        }
    }

    best.map(|(_, p)| p)
}

/// Detecta FSR e backends graficos na pasta do jogo.
/// Retorna (fsr_presente, dx12, vulkan).
pub fn detect_fsr(mod_dir: &Path, install_dir: &Path) -> (bool, bool, bool) {
    let mut fsr = false;
    let mut dx12 = false;
    let mut vk = false;
    // Pastas de assets que nao contem DLLs de runtime (evita varredura pesada).
    let skip_dirs = [
        "content", "paks", "movies", "saved", "shadercache", "cache", "extras",
    ];
    for dir in [mod_dir, install_dir] {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    !skip_dirs.contains(&n.as_str())
                } else {
                    true
                }
            })
            .flatten()
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let n = entry.file_name().to_string_lossy().to_lowercase();
            if n.contains("fidelityfx") || n.starts_with("ffx") || n.contains("fsr") {
                fsr = true;
                if n.contains("dx12") || n.contains("d3d12") {
                    dx12 = true;
                }
                if n.contains("_vk") || n.contains("vulkan") {
                    vk = true;
                }
            }
            if n == "amd_fidelityfx_dx12.dll" || n == "amd_fidelityfx_loader_dx12.dll" {
                dx12 = true;
            }
        }
        if fsr && dx12 {
            break;
        }
    }
    (fsr, dx12, vk)
}

/// Detecta anti-cheat conhecido pelos arquivos na pasta.
pub fn detect_anti_cheat(install_dir: &Path) -> Option<String> {
    if !install_dir.exists() {
        return None;
    }
    let patterns: [(&str, &str); 6] = [
        ("easyanticheat", "Easy Anti-Cheat"),
        ("eac_", "Easy Anti-Cheat"),
        ("battleye", "BattlEye"),
        ("beservice", "BattlEye"),
        ("bedaisy", "BattlEye"),
        ("vgc.exe", "Riot Vanguard"),
    ];
    for entry in WalkDir::new(install_dir).max_depth(4).into_iter().flatten() {
        let n = entry.file_name().to_string_lossy().to_lowercase();
        if n == MANIFEST_FILE_NAME {
            continue;
        }
        for (needle, label) in patterns {
            if n.contains(needle) {
                return Some(label.to_string());
            }
        }
    }
    None
}

/// Reavalia o estado de instalacao do mod para um jogo.
pub fn refresh_install_state(game: &mut Game, paths: &Paths) -> Result<()> {
    game.mod_installed = false;
    game.mod_version = None;
    game.divergence = false;
    game.last_checked = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());

    let Some(manifest) = load_manifest_for_game(game, paths) else {
        return Ok(());
    };

    game.mod_installed = true;
    game.mod_version = Some(manifest.mod_version.clone());

    // Divergencia: algum arquivo criado pelo mod sumiu ou foi trocado?
    let mod_dir = PathBuf::from(&game.mod_dir.clone().unwrap_or(manifest.mod_dir.clone()));
    let mut missing = 0usize;
    for rec in &manifest.created_files {
        let p = mod_dir.join(&rec.path);
        if !p.exists() {
            missing += 1;
            continue;
        }
        if !rec.sha256.is_empty() {
            let got = util::hash_file(&p);
            if !got.is_empty() && !got.eq_ignore_ascii_case(&rec.sha256) {
                missing += 1;
            }
        }
    }
    if missing > 0 {
        game.divergence = true;
        game.status_message =
            Some(format!("{missing} arquivo(s) do mod ausentes/alterados. Reaplique."));
    } else {
        game.status_message = Some("Mod ativo e integro.".into());
    }
    Ok(())
}

/// Carrega o manifesto do jogo (pasta do jogo primeiro, depois AppData).
pub fn load_manifest_for_game(game: &Game, paths: &Paths) -> Option<Manifest> {
    if let Some(mod_dir) = &game.mod_dir {
        let p = Path::new(mod_dir).join(MANIFEST_FILE_NAME);
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(m) = serde_json::from_str::<Manifest>(&text) {
                return Some(m);
            }
        }
    }
    let p = paths.manifest_file(&game.id);
    std::fs::read_to_string(p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
}
