//! Local/jar-based mod resolution.
//!
//! Resolves mods that cannot (or should not) be looked up through the
//! Modrinth / CurseForge APIs — either because the API is down (CurseForge
//! edge returning empty-body 403s), or because the mod is a custom bundle
//! served from an arbitrary URL (e.g. a Gitea/GitHub-hosted jar) that is not
//! published on either store.
//!
//! Strategy: download the jar once, extract the embedded mod metadata
//! (`META-INF/neoforge.mods.toml`, `META-INF/mods.toml`, or `fabric.mod.json`),
//! and cache the extracted metadata as a TOML file keyed by the file's content
//! hash. The jar itself is deleted after extraction. A later run with the same
//! hash reuses the cached TOML without re-downloading — until the mod is
//! updated (its hash changes), which triggers a fresh download.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::object::PackMod;

/// Metadata extracted from a mod jar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProject {
  pub name: String,
  pub description: String,
  pub authors: String,
  /// Best available link: the mod's homepage when the jar declares one,
  /// otherwise the download URL it was fetched from.
  pub url: String,
  pub version: String,
}

/// Resolve a mod's metadata from a jar, downloading and caching as needed.
///
/// * `url` — the URL to download the jar from.
/// * `cache_dir` — directory holding `<hash>.toml` (metadata) and, transiently,
///   `<hash>.jar` (the downloaded archive, deleted after extraction).
/// * `no_download` — if true, only use a cached metadata file; never hit the
///   network.
pub async fn resolve_local(
  pack_mod: &PackMod,
  url: &str,
  cache_dir: &Path,
  no_download: bool,
) -> Option<LocalProject> {
  let hash = pack_mod.hash().clone();
  let meta_path: PathBuf = cache_dir.join(format!("{hash}.toml"));

  // Cache hit: reuse the extracted metadata, no network.
  if meta_path.exists() {
    if let Ok(content) = std::fs::read_to_string(&meta_path) {
      if let Ok(project) = toml::from_str::<LocalProject>(&content) {
        return Some(project);
      }
    }
  }

  if no_download {
    return None;
  }

  if std::fs::create_dir_all(cache_dir).is_err() {
    return None;
  }

  let jar_path: PathBuf = cache_dir.join(format!("{hash}.jar"));

  if !download(url, &jar_path).await {
    let _ = std::fs::remove_file(&jar_path);
    return None;
  }

  let project = extract_metadata(&jar_path, url);

  // Cache the extracted metadata and drop the jar to limit bandwidth.
  if let Some(project) = &project {
    if let Ok(contents) = toml::to_string(project) {
      let _ = std::fs::write(&meta_path, contents);
    }
  }
  let _ = std::fs::remove_file(&jar_path);

  project
}

async fn download(url: &str, path: &Path) -> bool {
  let response = match reqwest::get(url).await {
    Ok(response) => response,
    Err(_) => return false,
  };

  if !response.status().is_success() {
    return false;
  }

  let bytes = match response.bytes().await {
    Ok(bytes) => bytes,
    Err(_) => return false,
  };

  std::fs::write(path, &bytes).is_ok()
}

/// Extract mod metadata from a jar (a zip archive), trying each metadata
/// location in priority order.
fn extract_metadata(jar_path: &Path, fallback_url: &str) -> Option<LocalProject> {
  let file = File::open(jar_path).ok()?;
  let mut archive = zip::ZipArchive::new(file).ok()?;

  // NeoForge / Forge metadata lives in META-INF.
  for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
    if let Ok(mut entry) = archive.by_name(name) {
      let mut content = String::new();
      if entry.read_to_string(&mut content).is_ok() {
        if let Some(project) = parse_forge_toml(&content, fallback_url) {
          return Some(project);
        }
      }
    }
  }

  // Fabric metadata is at the jar root.
  if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
    let mut content = String::new();
    if entry.read_to_string(&mut content).is_ok() {
      if let Some(project) = parse_fabric_json(&content, fallback_url) {
        return Some(project);
      }
    }
  }

  None
}

fn parse_forge_toml(content: &str, fallback_url: &str) -> Option<LocalProject> {
  let value: toml::Value = content.parse().ok()?;
  let mods = value.get("mods")?.as_array()?;
  let entry = mods.first()?.as_table()?;

  let name = entry
    .get("displayName")
    .or_else(|| entry.get("modId"))?
    .as_str()?
    .to_string();
  let description = entry
    .get("description")
    .and_then(|d| d.as_str())
    .unwrap_or("")
    .trim()
    .to_string();
  let authors = entry
    .get("authors")
    .and_then(|a| a.as_str())
    .unwrap_or("")
    .to_string();
  let version = entry
    .get("version")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();

  Some(LocalProject {
    name,
    description,
    authors,
    url: fallback_url.to_string(),
    version,
  })
}

fn parse_fabric_json(content: &str, fallback_url: &str) -> Option<LocalProject> {
  let value: serde_json::Value = serde_json::from_str(content).ok()?;

  let name = value
    .get("name")
    .or_else(|| value.get("id"))?
    .as_str()?
    .to_string();
  let description = value
    .get("description")
    .and_then(|d| d.as_str())
    .unwrap_or("")
    .to_string();
  let authors = value
    .get("authors")
    .and_then(|a| a.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>()
        .join(", ")
    })
    .unwrap_or_default();
  let version = value
    .get("version")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let url = value
    .get("contact")
    .and_then(|c| c.get("homepage"))
    .and_then(|h| h.as_str())
    .unwrap_or(fallback_url)
    .to_string();

  Some(LocalProject {
    name,
    description,
    authors,
    url,
    version,
  })
}
