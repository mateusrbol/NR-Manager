use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, Result};
use crate::logging::Logger;
use crate::models::{CachedRelease, Progress, Release, ReleaseAsset, ReleaseFeed};
use crate::paths::Paths;
use crate::settings::{Settings, SETUP_ASSET_NAME};
use crate::util;

/// Cache em memoria do feed de releases.
pub struct FeedCache {
    pub at: Instant,
    pub feed: ReleaseFeed,
}

const FETCH_TTL: Duration = Duration::from_secs(300);

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
    browser_download_url: String,
    #[serde(default)]
    download_count: u64,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

fn feed_file(paths: &Paths, settings: &Settings) -> PathBuf {
    paths.cache_dir(settings).join("github_releases.json")
}
fn etag_file(paths: &Paths, settings: &Settings) -> PathBuf {
    paths.cache_dir(settings).join("github_releases.etag")
}

fn parse_releases(json: &str) -> Result<Vec<Release>> {
    let raw: Vec<GhRelease> = serde_json::from_str(json)?;
    Ok(raw
        .into_iter()
        .map(|r| Release {
            tag: r.tag_name,
            name: r.name.unwrap_or_default(),
            body: r.body.unwrap_or_default(),
            published_at: r.published_at.unwrap_or_default(),
            prerelease: r.prerelease,
            draft: r.draft,
            html_url: r.html_url,
            assets: r
                .assets
                .into_iter()
                .map(|a| ReleaseAsset {
                    name: a.name,
                    size: a.size,
                    digest: a.digest,
                    download_url: a.browser_download_url,
                    download_count: a.download_count,
                })
                .collect(),
        })
        .collect())
}

/// Retorna (release, origem). `origin` = live | cache.
pub async fn fetch_releases(
    client: &reqwest::Client,
    paths: &Paths,
    settings: &Settings,
    logger: &Logger,
    _force: bool,
) -> Result<(Vec<Release>, String, Option<String>)> {
    std::fs::create_dir_all(paths.cache_dir(settings))?;

    let cached_disk = std::fs::read_to_string(feed_file(paths, settings)).ok();

    let (owner, repo) = settings
        .repo_parts()
        .ok_or_else(|| AppError::msg("URL do repositorio invalida."))?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=30");

    let mut req = client
        .get(&url)
        .header("User-Agent", "NR-Manager/0.1")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");

    if let Ok(etag) = std::fs::read_to_string(etag_file(paths, settings)) {
        let etag = etag.trim();
        if !etag.is_empty() {
            req = req.header("If-None-Match", etag);
        }
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Sem conexao com o GitHub ({e}).");
            logger.warn(&msg);
            if let Some(text) = cached_disk {
                if let Ok(list) = parse_releases(&text) {
                    if !list.is_empty() {
                        return Ok((list, "cache".into(), Some(msg)));
                    }
                }
            }
            return Err(AppError::msg(msg));
        }
    };

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_MODIFIED {
        // 304 = o GitHub confirmou que nosso conteudo esta atualizado; seguimos online.
        logger.info("Feed do GitHub nao mudou (304). Cache em dia.");
        let text = cached_disk.ok_or_else(|| AppError::msg("Cache ausente apos 304."))?;
        let list = parse_releases(&text)?;
        return Ok((list, "live".into(), None));
    }

    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let reset = resp
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let msg = match reset.and_then(|r| r.parse::<i64>().ok()) {
            Some(ts) => format!("Limite de requisicoes do GitHub atingido. Tente apos o epoch {ts}."),
            None => "Limite de requisicoes do GitHub atingido.".to_string(),
        };
        logger.warn(&msg);
        if let Some(text) = cached_disk {
            if let Ok(list) = parse_releases(&text) {
                if !list.is_empty() {
                    return Ok((list, "cache".into(), Some(msg)));
                }
            }
        }
        return Err(AppError::msg(msg));
    }

    if !status.is_success() {
        let msg = format!("GitHub respondeu {status}.");
        logger.warn(&msg);
        return Err(AppError::msg(msg));
    }

    let etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let text = resp.text().await?;

    let list = parse_releases(&text)?;
    let _ = std::fs::write(feed_file(paths, settings), &text);
    if let Some(etag) = etag {
        let _ = std::fs::write(etag_file(paths, settings), etag);
    }
    logger.info(format!("Lista de releases atualizada ({} versoes).", list.len()));
    Ok((list, "live".into(), None))
}

/// Monta um ReleaseFeed aplicando cache em memoria.
pub async fn get_feed(
    client: &reqwest::Client,
    paths: &Paths,
    settings: &Settings,
    logger: &Logger,
    cache: &parking_lot::Mutex<Option<FeedCache>>,
    force: bool,
) -> Result<ReleaseFeed> {
    if !force {
        let guard = cache.lock();
        if let Some(c) = guard.as_ref() {
            if c.at.elapsed() < FETCH_TTL {
                return Ok(c.feed.clone());
            }
        }
    }

    let (mut releases, origin, message) =
        fetch_releases(client, paths, settings, logger, force).await?;

    // Ordena por data de publicacao (mais recente primeiro).
    releases.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    let pinned = settings.pinned_version.clone();
    let latest = if let Some(pin) = pinned {
        releases.iter().find(|r| r.tag == pin).cloned()
    } else {
        releases
            .iter()
            .find(|r| !r.draft && !r.prerelease)
            .or_else(|| releases.iter().find(|r| !r.draft))
            .cloned()
    };

    let feed = ReleaseFeed {
        releases,
        latest,
        origin,
        message,
    };

    *cache.lock() = Some(FeedCache {
        at: Instant::now(),
        feed: feed.clone(),
    });
    Ok(feed)
}

/// Encontra o asset do setup em uma release.
pub fn setup_asset(release: &Release) -> Option<&ReleaseAsset> {
    release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(SETUP_ASSET_NAME))
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|a| a.name.to_lowercase().contains("setup"))
        })
}

/// Baixa (ou reutiliza do cache) o setup de uma release.
pub async fn download_setup(
    app: &AppHandle,
    client: &reqwest::Client,
    paths: &Paths,
    settings: &Settings,
    logger: &Logger,
    release: &Release,
) -> Result<CachedRelease> {
    let asset = setup_asset(release)
        .ok_or(AppError::NoAsset)?
        .clone();
    let dir = paths.releases_dir(settings).join(sanitize_tag(&release.tag));
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(&asset.name);

    let expected = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:").map(|s| s.to_lowercase()));

    // Reutiliza se ja existe e o hash confere.
    if target.exists() {
        if let Some(exp) = &expected {
            let got = util::hash_file(&target);
            if got.eq_ignore_ascii_case(exp) {
                logger.info(format!("Setup {} ja esta no cache.", release.tag));
                return write_cached_meta(paths, settings, &dir, release, &target, Some(got), "github")
                    .await;
            }
        } else {
            logger.info(format!("Setup {} ja esta no cache (sem hash de referencia).", release.tag));
            return write_cached_meta(paths, settings, &dir, release, &target, None, "github").await;
        }
    }

    logger.info(format!(
        "Baixando setup {} ({})...",
        release.tag,
        util::human_bytes(asset.size)
    ));

    let tmp = dir.join(format!("{}.download", asset.name));
    let resp = client
        .get(&asset.download_url)
        .header("User-Agent", "NR-Manager/0.1")
        .send()
        .await?
        .error_for_status()?;

    let total = resp.content_length().unwrap_or(asset.size);
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        let percent = if total > 0 {
            Some((downloaded as f64 / total as f64) * 100.0)
        } else {
            None
        };
        let _ = app.emit(
            "nr-progress",
            Progress {
                game_id: None,
                stage: "download".into(),
                message: format!(
                    "Baixando {} - {} / {}",
                    release.tag,
                    util::human_bytes(downloaded),
                    util::human_bytes(total)
                ),
                percent,
            },
        );
    }
    file.flush().await?;
    drop(file);

    if target.exists() {
        let _ = std::fs::remove_file(&target);
    }
    tokio::fs::rename(&tmp, &target).await?;

    let got = util::hash_file(&target);
    if let Some(exp) = &expected {
        if !got.eq_ignore_ascii_case(exp) {
            let _ = std::fs::remove_file(&target);
            return Err(AppError::msg(format!(
                "Hash do setup nao confere para {}. Download corrompido; removido.",
                release.tag
            )));
        }
    }
    logger.info(format!("Setup {} baixado com sucesso.", release.tag));
    write_cached_meta(paths, settings, &dir, release, &target, Some(got), "github").await
}

async fn write_cached_meta(
    _paths: &Paths,
    _settings: &Settings,
    dir: &Path,
    release: &Release,
    target: &Path,
    sha256: Option<String>,
    source: &str,
) -> Result<CachedRelease> {
    let size = std::fs::metadata(target).map(|m| m.len()).unwrap_or(0);
    let cached = CachedRelease {
        tag: release.tag.clone(),
        name: if release.name.is_empty() {
            release.tag.clone()
        } else {
            release.name.clone()
        },
        path: util::to_win_string(target),
        size,
        sha256,
        source: source.into(),
        downloaded_at: chrono::Utc::now().to_rfc3339(),
        body: Some(release.body.clone()),
        published_at: Some(release.published_at.clone()),
    };
    let meta = dir.join("release.json");
    std::fs::write(meta, serde_json::to_string_pretty(&cached)?)?;
    Ok(cached)
}

/// Importa um setup local (exe ou zip) como "versao personalizada".
pub async fn import_custom_setup(
    paths: &Paths,
    settings: &Settings,
    logger: &Logger,
    source_path: &Path,
) -> Result<CachedRelease> {
    if !source_path.exists() {
        return Err(AppError::NotFound(util::to_win_string(source_path)));
    }
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let tag = format!("custom-{stamp}");
    let dir = paths.releases_dir(settings).join(&tag);
    std::fs::create_dir_all(&dir)?;

    let ext = source_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let target = if ext == "zip" {
        // Extrai o zip e localiza o setup dentro.
        let file = std::fs::File::open(source_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::msg(format!("Zip invalido: {e}")))?;
        let extract_dir = dir.join("extracted");
        std::fs::create_dir_all(&extract_dir)?;
        archive
            .extract(&extract_dir)
            .map_err(|e| AppError::msg(format!("Falha ao extrair zip: {e}")))?;
        find_setup_inside(&extract_dir).ok_or_else(|| {
            AppError::msg(format!(
                "Nenhum {SETUP_ASSET_NAME} (ou setup .exe) encontrado dentro do zip."
            ))
        })?
    } else if ext == "7z" || ext == "rar" {
        return Err(AppError::msg(
            "Formatos .7z/.rar nao sao suportados. Extraia o setup.exe e importe o .exe (ou use .zip).",
        ));
    } else {
        let file_name = source_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| SETUP_ASSET_NAME.to_string());
        let dst = dir.join(&file_name);
        std::fs::copy(source_path, &dst)?;
        dst
    };

    let cached = CachedRelease {
        tag: tag.clone(),
        name: format!("Versao personalizada ({stamp})"),
        path: util::to_win_string(&target),
        size: std::fs::metadata(&target)?.len(),
        sha256: Some(util::hash_file(&target)),
        source: "custom".into(),
        downloaded_at: chrono::Utc::now().to_rfc3339(),
        body: Some("Setup importado manualmente (build antecipada / Discord).".into()),
        published_at: None,
    };
    std::fs::write(dir.join("release.json"), serde_json::to_string_pretty(&cached)?)?;
    logger.info(format!(
        "Setup personalizado importado: {}",
        util::to_win_string(&target)
    ));
    Ok(cached)
}

/// Procura o setup dentro de uma pasta extraida.
fn find_setup_inside(dir: &Path) -> Option<PathBuf> {
    let mut any_setup: Option<PathBuf> = None;
    for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name == SETUP_ASSET_NAME.to_lowercase() {
            return Some(entry.path().to_path_buf());
        }
        if name.ends_with(".exe") && name.contains("setup") && any_setup.is_none() {
            any_setup = Some(entry.path().to_path_buf());
        }
    }
    any_setup
}

/// Lista as releases presentes no cache local.
pub fn list_cached(paths: &Paths, settings: &Settings) -> Vec<CachedRelease> {
    let dir = paths.releases_dir(settings);
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let meta = entry.path().join("release.json");
        if let Ok(text) = std::fs::read_to_string(meta) {
            if let Ok(c) = serde_json::from_str::<CachedRelease>(&text) {
                out.push(c);
            }
        }
    }
    out.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));
    out
}

/// Remove releases personalizadas do cache.
pub fn remove_cached(paths: &Paths, settings: &Settings, tag: &str) -> Result<()> {
    let dir = paths.releases_dir(settings).join(sanitize_tag(tag));
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

fn sanitize_tag(tag: &str) -> String {
    tag.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}
