//! Keep generated cache files out of git and out of the packwiz index.
//!
//! packwizml produces two local, regenerable artifacts: the metadata cache
//! directory (`--cache-dir`, default `.cache`) and the modlist cache file
//! (`--cache`, default `.packwiz-modlist.cache.json`). On every run we make
//! sure both are ignored — in `.gitignore` for git and in `.packwizignore` for
//! packwiz's index/refresh. The operation is idempotent: an entry is appended
//! only if it isn't already present, and the files are created if needed.

use std::path::Path;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::Args;
use crate::error::GlobalResult;

/// Ensure `.gitignore` and `.packwizignore` at the pack root ignore the cache
/// artifacts. Non-destructive: only adds missing entries, never rewrites or
/// reorders existing content.
pub async fn ensure_ignored(args: &Args) -> GlobalResult<()> {
  let entries = wanted_entries(args);

  for filename in [".gitignore", ".packwizignore"] {
    let path = args.path.join(filename);
    let mut content = read_optional(&path).await?;
    let mut changed = false;

    for entry in &entries {
      if entry.is_empty() {
        continue;
      }
      let present = content.lines().any(|line| line.trim_end() == entry.as_str());
      if !present {
        if !content.is_empty() && !content.ends_with('\n') {
          content.push('\n');
        }
        content.push_str(entry);
        content.push('\n');
        changed = true;
      }
    }

    if changed {
      let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .await?;
      file.write_all(content.as_bytes()).await?;
      file.flush().await?;
      file.sync_all().await?;
      log::info!("Added {} to {}", entries.join(", "), filename);
    }
  }

  Ok(())
}

fn wanted_entries(args: &Args) -> Vec<String> {
  let mut entries = Vec::with_capacity(2);

  let dir = args
    .cache_dir
    .to_string_lossy()
    .trim_end_matches('/')
    .to_string();
  if !dir.is_empty() {
    entries.push(dir);
  }

  if let Some(name) = args.cache.file_name() {
    let name = name.to_string_lossy().into_owned();
    if !name.is_empty() {
      entries.push(name);
    }
  }

  entries
}

async fn read_optional(path: &Path) -> GlobalResult<String> {
  match OpenOptions::new().read(true).open(path).await {
    Ok(mut file) => {
      let mut content = String::new();
      file.read_to_string(&mut content).await?;
      Ok(content)
    }
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
    Err(e) => Err(e.into()),
  }
}
