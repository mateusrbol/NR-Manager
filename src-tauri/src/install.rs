use std::path::{Path, PathBuf};
use std::process::Command;

use sysinfo::System;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, Result};
use crate::games;
use crate::github;
use crate::manifest;
use crate::models::{Game, GameStatus, Progress, RepairReport};
use crate::paths::manifest_in_game_dir;
use crate::settings::{Settings, DLSSNR_DLL_NAME, MANIFEST_FILE_NAME, SETUP_ASSET_NAME};
use crate::state::{save_json, AppState};
use crate::util;

/// Origem do setup a ser executado.
pub struct SetupSource {
    pub tag: String,
    pub path: PathBuf,
    pub sha256: Option<String>,
}

fn emit(app: &AppHandle, game_id: Option<String>, stage: &str, message: impl Into<String>, percent: Option<f64>) {
    let _ = app.emit(
        "nr-progress",
        Progress {
            game_id,
            stage: stage.into(),
            message: message.into(),
            percent,
        },
    );
}

pub fn find_game(state: &AppState, id: &str) -> Result<Game> {
    state
        .library
        .lock()
        .iter()
        .find(|g| g.id == id)
        .cloned()
        .ok_or_else(|| AppError::msg("Jogo nao encontrado na biblioteca."))
}

pub fn upsert(state: &AppState, game: Game) {
    let mut lib = state.library.lock();
    if let Some(existing) = lib.iter_mut().find(|g| g.id == game.id) {
        *existing = game;
    } else {
        lib.push(game);
    }
}

/// Verifica se o jogo esta em execucao (arquivo em uso).
pub fn is_game_running(game: &Game) -> bool {
    let sys = System::new_all();
    let install = util::norm_path(Path::new(&game.install_dir));
    let exe_name = game
        .exe_path
        .as_ref()
        .and_then(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_lowercase());

    for process in sys.processes().values() {
        if let Some(exe) = process.exe() {
            let norm = util::norm_path(exe);
            if !install.is_empty() && norm.starts_with(&install) {
                return true;
            }
        }
        if let Some(name) = &exe_name {
            let pname = process.name().to_string_lossy().to_lowercase();
            if &pname == name {
                return true;
            }
        }
    }
    false
}

/// Verifica se o processo atual tem privilegios elevados.
pub fn is_elevated() -> bool {
    let script = "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)";
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Reinicia o aplicativo solicitando elevacao (UAC).
pub fn restart_elevated() -> Result<()> {
    let exe = std::env::current_exe()?;
    let script = format!(
        "$p = Start-Process -FilePath '{}' -Verb RunAs -PassThru; exit $p.Id",
        escape_ps(&exe.to_string_lossy())
    );
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .spawn()?;
    Ok(())
}

/// Executa o setup elevado e aguarda o termino. Retorna o exit code.
fn run_setup(setup: &Path, workdir: &Path, silent_args: &str) -> Result<i32> {
    let args = if silent_args.trim().is_empty() {
        String::new()
    } else {
        format!(" -ArgumentList '{}'", escape_ps(silent_args.trim()))
    };
    let script = format!(
        "$ErrorActionPreference='Stop'\r\ntry {{\r\n  $p = Start-Process -FilePath '{exe}' -WorkingDirectory '{cwd}'{args} -Verb RunAs -Wait -PassThru\r\n  exit $p.ExitCode\r\n}} catch {{\r\n  Write-Error $_\r\n  exit 1223\r\n}}\r\n",
        exe = escape_ps(&setup.to_string_lossy()),
        cwd = escape_ps(&workdir.to_string_lossy()),
        args = args
    );

    let tmp = std::env::temp_dir().join(format!(
        "nrmanager-setup-{}.ps1",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    ));
    // UTF-8 com BOM para o PowerShell 5.1 ler caminhos acentuados corretamente.
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(script.as_bytes());
    std::fs::write(&tmp, content)?;

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &tmp.to_string_lossy(),
        ])
        .status()?;
    let _ = std::fs::remove_file(&tmp);

    Ok(status.code().unwrap_or(-1))
}

fn escape_ps(s: &str) -> String {
    s.replace('\'', "''")
}

/// Resolve o caminho do setup a usar (cache, pin, custom ou download).
pub async fn resolve_setup(
    app: &AppHandle,
    state: &AppState,
    tag: Option<String>,
) -> Result<SetupSource> {
    let settings = state.settings();

    // Setup local configurado explicitamente nas Configuracoes.
    if tag.is_none() && !settings.custom_setup_path.trim().is_empty() {
        let p = PathBuf::from(settings.custom_setup_path.trim());
        if p.exists() {
            return Ok(SetupSource {
                tag: "custom-local".into(),
                sha256: Some(util::hash_file(&p)),
                path: p,
            });
        }
    }

    let wanted = tag
        .clone()
        .or_else(|| settings.pinned_version.clone());

    // 1) Ja esta no cache?
    let cached = github::list_cached(&state.paths, &settings);
    if let Some(w) = &wanted {
        if let Some(c) = cached.iter().find(|c| &c.tag == w) {
            let p = PathBuf::from(&c.path);
            if p.exists() {
                return Ok(SetupSource {
                    tag: c.tag.clone(),
                    path: p,
                    sha256: c.sha256.clone(),
                });
            }
        }
    }

    // 2) Busca a release (respeitando pin/tag) e baixa.
    let feed = github::get_feed(
        &state.client,
        &state.paths,
        &settings,
        &state.logger,
        &state.feed,
        false,
    )
    .await?;

    let release = match &wanted {
        Some(w) => feed.releases.iter().find(|r| &r.tag == w).cloned(),
        None => feed.latest.clone(),
    }
    .ok_or_else(|| {
        AppError::msg(
            "Nenhuma release encontrada. Verifique a internet ou importe um setup local.",
        )
    })?;

    let cached =
        github::download_setup(app, &state.client, &state.paths, &settings, &state.logger, &release)
            .await?;

    Ok(SetupSource {
        tag: cached.tag,
        path: PathBuf::from(cached.path),
        sha256: cached.sha256,
    })
}

/// Encontra o nvngx_dlssnr.dll fornecido pelo usuario.
pub fn resolve_dll(settings: &Settings) -> Result<PathBuf> {
    let raw = settings.dlssnr_dll_path.trim();
    if raw.is_empty() {
        return Err(AppError::msg(format!(
            "Informe o caminho do {DLSSNR_DLL_NAME} em Configuracoes (voce obtem o arquivo de um jogo com DLSS 5)."
        )));
    }
    let path = PathBuf::from(raw);
    if !path.exists() {
        return Err(AppError::NotFound(raw.to_string()));
    }
    Ok(path)
}

/// Aplica o mod em um jogo.
pub async fn apply_mod(
    app: &AppHandle,
    state: &AppState,
    game_id: &str,
    tag: Option<String>,
) -> Result<Game> {
    let mut game = find_game(state, game_id)?;
    let settings = state.settings();
    let logger = state.logger.clone();

    let mod_dir = game
        .mod_dir
        .clone()
        .ok_or_else(|| AppError::msg("Pasta do mod nao definida. Corrija o caminho do executavel."))?;
    let mod_dir_p = PathBuf::from(&mod_dir);
    if !mod_dir_p.exists() {
        return Err(AppError::NotFound(mod_dir.clone()));
    }
    let exe = game
        .exe_path
        .clone()
        .ok_or_else(|| AppError::msg("Executavel do jogo nao localizado. Corrija na tela do jogo."))?;
    if !Path::new(&exe).exists() {
        return Err(AppError::NotFound(exe));
    }
    if is_game_running(&game) {
        return Err(AppError::msg(
            "O jogo esta aberto (arquivo em uso). Feche-o antes de aplicar o mod.",
        ));
    }

    // Pre-requisitos.
    let dll_src = resolve_dll(&settings)?;
    let source = resolve_setup(app, state, tag).await?;

    // Se ja houver instalacao anterior, desfaz primeiro.
    if game.mod_installed {
        logger.info("Desfazendo instalacao anterior antes de reaplicar...");
        let _ = uninstall(app, state, game_id).await;
        game = find_game(state, game_id)?;
    }

    let cap = settings.large_cap_bytes();

    emit(
        app,
        Some(game.id.clone()),
        "snapshot",
        "Analisando pasta do jogo (antes)...",
        None,
    );
    let before = manifest::snapshot(&mod_dir_p, cap, &|_| {})?;

    emit(
        app,
        Some(game.id.clone()),
        "backup",
        "Criando backup dos arquivos originais...",
        None,
    );
    let backup_dir = state.paths.backup_dir_for(&game.id);
    let backup = manifest::make_backup(&before, &mod_dir_p, &backup_dir, cap, &logger)?;

    // Copia setup + DLL para a pasta do jogo (fluxo manual do mod).
    let setup_name = source
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| SETUP_ASSET_NAME.to_string());
    let setup_dst = mod_dir_p.join(&setup_name);
    let dll_dst = mod_dir_p.join(DLSSNR_DLL_NAME);

    emit(
        app,
        Some(game.id.clone()),
        "prepare",
        format!("Copiando {} e {}...", setup_name, DLSSNR_DLL_NAME),
        None,
    );
    std::fs::copy(&source.path, &setup_dst).map_err(|e| {
        AppError::Permission(format!(
            "Nao foi possivel copiar o setup para a pasta do jogo ({e}). Rode o app como administrador."
        ))
    })?;
    std::fs::copy(&dll_src, &dll_dst).map_err(|e| {
        AppError::Permission(format!(
            "Nao foi possivel copiar o {DLSSNR_DLL_NAME} ({e}). Rode o app como administrador."
        ))
    })?;

    // Executa o instalador (interativo/elevado, aguardando o termino).
    emit(
        app,
        Some(game.id.clone()),
        "setup",
        "Aguardando o instalador do mod terminar. Siga as instrucoes na janela aberta.",
        None,
    );
    let code = run_setup(&setup_dst, &mod_dir_p, &settings.setup_silent_args)?;
    logger.info(format!(
        "Instalador '{}' finalizado com codigo {code}.",
        source.tag
    ));

    emit(
        app,
        Some(game.id.clone()),
        "snapshot",
        "Analisando pasta do jogo (depois)...",
        None,
    );
    let manifest = manifest::build_manifest(
        &game.id,
        &game.name,
        &source.tag,
        source.sha256.clone(),
        &mod_dir_p,
        &backup_dir,
        cap,
        &before,
        &backup,
    )?;

    // Grava manifestos (pasta do jogo + AppData).
    save_json(&manifest_in_game_dir(&mod_dir_p), &manifest)?;
    save_json(&state.paths.manifest_file(&game.id), &manifest)?;

    if code != 0 {
        logger.error(format!(
            "Falha na instalacao (codigo {code}). Revertendo alteracoes..."
        ));
        let _ = manifest::uninstall(&manifest, &logger);
        let _ = std::fs::remove_file(state.paths.manifest_file(&game.id));
        let _ = std::fs::remove_dir_all(&backup_dir);
        return Err(AppError::msg(if code == 1223 {
            "Operacao cancelada no UAC. O mod nao foi aplicado e as alteracoes foram revertidas."
                .to_string()
        } else {
            format!("O instalador falhou (codigo {code}). As alteracoes foram revertidas.")
        }));
    }

    game.mod_installed = true;
    game.mod_version = Some(source.tag.clone());
    game.divergence = false;
    game.status_message = Some("Mod ativo e integro.".into());
    game.last_checked = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    upsert(state, game.clone());
    state.save_library()?;

    emit(
        app,
        Some(game.id.clone()),
        "done",
        format!("Mod {} aplicado em {}.", source.tag, game.name),
        Some(100.0),
    );
    logger.info(format!("Mod {} aplicado em '{}'.", source.tag, game.name));
    Ok(game)
}

/// Remove o mod de um jogo, restaurando o estado original.
pub async fn uninstall(app: &AppHandle, state: &AppState, game_id: &str) -> Result<Game> {
    let mut game = find_game(state, game_id)?;
    let logger = state.logger.clone();

    let manifest = games::load_manifest_for_game(&game, &state.paths)
        .ok_or_else(|| AppError::msg("Nao ha manifesto de instalacao para este jogo."))?;

    if is_game_running(&game) {
        return Err(AppError::msg(
            "O jogo esta aberto (arquivo em uso). Feche-o antes de remover o mod.",
        ));
    }

    emit(
        app,
        Some(game.id.clone()),
        "remove",
        "Removendo arquivos do mod e restaurando originais...",
        None,
    );
    manifest::uninstall(&manifest, &logger)?;

    let backup_dir = state.paths.backup_dir_for(&game.id);
    let _ = std::fs::remove_dir_all(&backup_dir);
    let _ = std::fs::remove_file(state.paths.manifest_file(&game.id));

    game.mod_installed = false;
    game.mod_version = None;
    game.divergence = false;
    game.status_message = Some("Mod removido. Pasta restaurada ao estado original.".into());
    game.last_checked = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    upsert(state, game.clone());
    state.save_library()?;

    emit(
        app,
        Some(game.id.clone()),
        "done",
        format!("Mod removido de {}.", game.name),
        Some(100.0),
    );
    logger.info(format!("Mod removido de '{}'.", game.name));
    Ok(game)
}

/// Verifica a integridade dos arquivos do mod (reparar).
pub fn repair(state: &AppState, game_id: &str) -> Result<RepairReport> {
    let game = find_game(state, game_id)?;
    let Some(manifest) = games::load_manifest_for_game(&game, &state.paths) else {
        return Ok(RepairReport {
            ok: false,
            message: "Nao ha instalacao registrada para este jogo.".into(),
            ..Default::default()
        });
    };

    let mod_dir = PathBuf::from(&manifest.mod_dir);
    let mut missing = Vec::new();
    let mut modified = Vec::new();

    for rec in &manifest.created_files {
        let p = mod_dir.join(&rec.path);
        if !p.exists() {
            missing.push(rec.path.clone());
            continue;
        }
        if !rec.sha256.is_empty() {
            let got = util::hash_file(&p);
            if !got.is_empty() && !got.eq_ignore_ascii_case(&rec.sha256) {
                modified.push(rec.path.clone());
            }
        }
    }

    let ok = missing.is_empty() && modified.is_empty();
    let message = if ok {
        format!("Integridade OK: {} arquivo(s) verificados.", manifest.created_files.len())
    } else {
        format!(
            "{} arquivo(s) ausentes e {} alterado(s). Reaplique o mod para corrigir.",
            missing.len(),
            modified.len()
        )
    };

    Ok(RepairReport {
        ok,
        checked: manifest.created_files.len(),
        missing,
        modified,
        message,
    })
}

/// Estado detalhado de instalacao de um jogo.
pub fn game_status(state: &AppState, game_id: &str) -> Result<GameStatus> {
    let game = find_game(state, game_id)?;
    let manifest = games::load_manifest_for_game(&game, &state.paths);
    let manifest_path = game
        .mod_dir
        .as_ref()
        .map(|d| util::to_win_string(&PathBuf::from(d).join(MANIFEST_FILE_NAME)));

    Ok(GameStatus {
        game_id: game.id.clone(),
        installed: manifest.is_some(),
        version: manifest.as_ref().map(|m| m.mod_version.clone()),
        installed_at: manifest.as_ref().map(|m| m.installed_at.clone()),
        files_count: manifest
            .as_ref()
            .map(|m| m.created_files.len() + m.modified_files.len())
            .unwrap_or(0),
        divergence: game.divergence,
        game_running: is_game_running(&game),
        manifest_path,
        backup_dir: manifest.as_ref().map(|m| m.backup_dir.clone()),
        message: game.status_message.clone().unwrap_or_default(),
    })
}
