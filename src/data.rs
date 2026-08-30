use std::collections::HashMap;
use std::path::{Path, PathBuf};

use itertools::Itertools;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::local::resolve_local;
use GlobalError::Validation;

use crate::cache::Cache;
use crate::error::ValidationError::{DirNotExist, MustBeDir, PackNotFound};
use crate::error::{GlobalError, GlobalResult};
use crate::object::{
  CurseForgeProject, CurseforgeModIds, CurseforgeMods, ModrinthProject, ModrinthTeamMember, Pack,
  PackMod, PackMods, Project,
};
use crate::Args;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const MODRINTH_API: &str = "https://api.modrinth.com/v2";

pub fn read_toml_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> GlobalResult<T> {
  let data = std::fs::read_to_string(path)?;

  toml::from_str::<T>(&data).map_err(GlobalError::from)
}

pub fn get_mods(args: &Args) -> GlobalResult<PackMods> {
  let path = if args.mods_custom {
    args.mods.clone()
  } else {
    args.path.join(&args.mods)
  };

  match () {
    _ if !path.exists() => Err(Validation(DirNotExist(path))),
    _ if !path.is_dir() => Err(Validation(MustBeDir(path))),
    _ => path
      .read_dir()?
      .filter_map(|it| it.ok())
      .filter(|it| it.file_name().to_string_lossy().ends_with(".toml"))
      .map(|it| read_toml_file(it.path()))
      .collect::<GlobalResult<PackMods>>(),
  }
}

pub fn get_data(args: &Args) -> GlobalResult<(Pack, PackMods)> {
  let path = args.path.clone();
  let pack = path.join("pack.toml");

  match () {
    _ if !path.exists() => Err(Validation(DirNotExist(path))),
    _ if !path.is_dir() => Err(Validation(MustBeDir(path))),
    _ if !pack.is_file() => Err(Validation(PackNotFound(path))),
    _ => {
      let mods = get_mods(args)?;
      let pack = read_toml_file(pack)?;

      Ok((pack, mods))
    }
  }
}

// #[allow(unused)]
// async fn request_modrinth_project(id: &str) -> GlobalResult<ModrinthProject> {
//   let url = format!("{MODRINTH_API}/project/{id}");
//   let response = reqwest::get(url).await?;
//   let project = response.json().await?;
//
//   Ok(project)
// }

async fn request_modrinth_teams(ids: Vec<String>) -> GlobalResult<Vec<Vec<ModrinthTeamMember>>> {
  let ids = serde_json::to_string(&ids)?;
  let url = format!("{MODRINTH_API}/teams?ids={ids}");
  let response = reqwest::get(url).await?;

  let team = response.json().await?;

  Ok(team)
}

async fn request_modrinth_projects(ids: Vec<String>) -> GlobalResult<Vec<ModrinthProject>> {
  let ids = serde_json::to_string(&ids)?;
  let url = format!("{MODRINTH_API}/projects?ids={ids}");
  let response = reqwest::get(url).await?;
  let projects: Vec<ModrinthProject> = response.json().await?;

  let mut teams_map = projects
    .into_iter()
    .into_group_map_by(|project| project.team.clone());

  let ids = teams_map.keys().cloned().collect_vec();
  let teams = request_modrinth_teams(ids).await?;

  for team in teams.iter() {
    for member in team.iter() {
      let projects = teams_map.get_mut(&member.team_id).unwrap();

      for project in projects {
        project.team_members.push(member.clone());
      }
    }
  }

  let projects = teams_map.into_values().flatten().collect_vec();

  Ok(projects)
}

// #[allow(unused)]
// async fn request_curseforge_project(id: u32) -> GlobalResult<CurseForgeProject> {
//   let url = format!("{CURSEFORGE_API}/mods/{id}");
//   let response = Client::builder()
//     .build()?
//     .get(url)
//     .header("x-api-key", CURSEFORGE_API_KEY)
//     .send()
//     .await?;
//
//   let project = response.json().await?;
//
//   Ok(project)
// }

async fn request_curseforge_projects(
  ids: Vec<u32>,
  api_key: &str,
) -> GlobalResult<Vec<CurseForgeProject>> {
  let ids = CurseforgeModIds { mod_ids: ids };
  let url = format!("{CURSEFORGE_API}/mods");
  let response = Client::builder()
    .build()?
    .post(url)
    .header("x-api-key", api_key)
    .json(&ids)
    .send()
    .await?;

  let projects: CurseforgeMods = response.json().await?;

  Ok(projects.data)
}

// #[allow(unused)]
// pub async fn request_project(pack_mod: &PackMod) -> GlobalResult<Project> {
//   if let Some(pack_mod) = &pack_mod.update.modrinth {
//     return request_modrinth_project(&pack_mod.mod_id)
//       .await
//       .map(Project::from);
//   }
//
//   if let Some(pack_mod) = &pack_mod.update.curseforge {
//     return request_curseforge_project(pack_mod.project_id)
//       .await
//       .map(Project::from);
//   }
//
//   unreachable!()
// }

pub async fn get_modrinth_projects(
  cache: &mut Cache,
  mods: &PackMods,
) -> GlobalResult<Vec<Project>> {
  let mut modrinth = Vec::with_capacity(mods.len());

  let filter = mods.iter().filter(|it| {
    it.update
      .as_ref()
      .and_then(|u| u.modrinth.as_ref())
      .is_some()
  });

  match cache.get_all(filter.clone()) {
    Some(projects) => modrinth.extend(projects.into_iter().cloned()),
    None => {
      let lookup = filter
        .clone()
        .into_group_map_by(|it| it.id())
        .into_iter()
        .map(|(key, value)| (key, value[0]))
        .collect::<HashMap<_, _>>();

      let modrinth_ids = filter.map(|it| it.id()).collect();
      let projects = request_modrinth_projects(modrinth_ids)
        .await?
        .into_iter()
        .map(|it| (lookup[&it.id], Project::from(it)));

      cache.insert_all(projects.clone());
      modrinth.extend(projects.map(|it| it.1));
    }
  };

  Ok(modrinth)
}

pub async fn get_curseforge_projects(
  cache: &mut Cache,
  mods: &PackMods,
  args: &Args,
) -> GlobalResult<Vec<Project>> {
  let mut curseforge = Vec::with_capacity(mods.len());

  let filter = mods.iter().filter(|it| {
    it.update
      .as_ref()
      .and_then(|u| u.curseforge.as_ref())
      .is_some()
  });

  match cache.get_all(filter.clone()) {
    Some(projects) => curseforge.extend(projects.into_iter().cloned()),
    None => {
      let lookup = filter
        .clone()
        .into_group_map_by(|it| it.id())
        .into_iter()
        .map(|(key, value)| (key, value[0]))
        .collect::<HashMap<_, _>>();

      let curseforge_ids: Vec<u32> = filter.filter_map(|it| it.id().parse().ok()).collect();

      if curseforge_ids.is_empty() {
        return Ok(curseforge);
      }

      // If the CurseForge API fails (e.g. the edge is returning empty-body 403s),
      // fall back to downloading the jar from the reconstructed CDN URL and
      // extracting its embedded metadata — only falling back to a bare numeric
      // placeholder if the download or extraction fails.
      match resolve_curseforge_projects(&lookup, curseforge_ids).await {
        Ok(projects) => {
          cache.insert_all(projects.clone());
          curseforge.extend(projects.into_iter().map(|it| it.1));
        }
        Err(_) => {
          if args.log_level >= log::LevelFilter::Warn {
            println!(
              "WARN: CurseForge API request failed; resolving {} CurseForge mod(s) from their jars.",
              lookup.len()
            );
          }
          for pack_mod in lookup.into_values() {
            match resolve_curseforge_from_jar(pack_mod, cache_dir(args)).await {
              Some(project) => curseforge.push(project),
              None => curseforge.push(placeholder_curseforge(pack_mod)),
            }
          }
        }
      }
    }
  };

  Ok(curseforge)
}

/// Reconstruct the CurseForge CDN URL for a mod file. The `.pw.toml` for a CF
/// mod stores no download URL (it uses `mode = "metadata:curseforge"`), so the
/// file is addressed directly on the CDN by file-id and filename.
fn curseforge_cdn_url(pack_mod: &PackMod) -> Option<String> {
  let cf = pack_mod.update.as_ref()?.curseforge.as_ref()?;
  let file_id = cf.file_id;
  Some(format!(
    "https://edge.forgecdn.net/files/{}/{}/{}",
    file_id / 1000,
    file_id % 1000,
    pack_mod.filename
  ))
}

fn cache_dir(args: &Args) -> PathBuf {
  if args.cache_dir_custom {
    args.cache_dir.clone()
  } else {
    args.path.join(&args.cache_dir)
  }
}

/// Try to resolve a CurseForge mod from its jar when the API is unavailable.
async fn resolve_curseforge_from_jar(pack_mod: &PackMod, cache_dir: PathBuf) -> Option<Project> {
  let url = curseforge_cdn_url(pack_mod)?;
  resolve_local(pack_mod, &url, &cache_dir, false)
    .await
    .map(Project::Local)
}

async fn resolve_curseforge_projects<'a>(
  lookup: &'a HashMap<String, &'a PackMod>,
  curseforge_ids: Vec<u32>,
) -> GlobalResult<Vec<(&'a PackMod, Project)>> {
  let api_key = std::env::var("CF_API_KEY").map_err(|_| {
    GlobalError::custom(
      "CurseForge",
      "CF_API_KEY is not set. Set it in the environment or a .env file to query CurseForge mods.",
    )
  })?;

  let projects = request_curseforge_projects(curseforge_ids, &api_key)
    .await?
    .into_iter()
    .map(|it| (lookup[&it.id.to_string()], Project::from(it)))
    .collect::<Vec<_>>();

  Ok(projects)
}

/// Build a placeholder `Project` for a CurseForge mod from the metadata already
/// present in the packwiz file, so output can be produced without querying the
/// (possibly unavailable) CurseForge API.
fn placeholder_curseforge(pack_mod: &PackMod) -> Project {
  let id = pack_mod.id();
  let name = pack_mod.name.clone();
  let id_num = id.parse::<u32>().unwrap_or(0);
  // The real slug is only available via the CurseForge API, so point the link at
  // the numeric project ID instead — CurseForge redirects /mc-mods/<id> to the slug.
  Project::CurseForge(CurseForgeProject {
    id: id_num,
    slug: id_num.to_string(),
    name,
    summary: String::from(
      "CurseForge mod — API unavailable, shown as a placeholder. See the CurseForge project page.",
    ),
    authors: Vec::new(),
    logo: None,
  })
}

pub async fn get_projects(
  cache: &mut Cache,
  mods: &PackMods,
  args: &Args,
) -> GlobalResult<Vec<Project>> {
  let mut projects = Vec::with_capacity(mods.len());

  let modrinth = get_modrinth_projects(cache, mods).await?;
  let curseforge = get_curseforge_projects(cache, mods, args).await?;
  let custom = get_custom_projects(mods, cache_dir(args), args.no_download).await;

  projects.extend_from_slice(&modrinth);
  projects.extend_from_slice(&curseforge);
  projects.extend_from_slice(&custom);

  Ok(projects)
}

/// Resolve "custom" mods — those published on neither Modrinth nor CurseForge,
/// carrying a direct `[download].url`. Their metadata is pulled from the jar
/// itself, cached locally, and reused across runs until the file hash changes.
async fn get_custom_projects(
  mods: &PackMods,
  cache_dir: PathBuf,
  no_download: bool,
) -> Vec<Project> {
  let mut projects = Vec::new();

  for pack_mod in mods {
    let is_custom = match &pack_mod.update {
      Some(update) => update.modrinth.is_none() && update.curseforge.is_none(),
      None => true,
    };

    if !is_custom {
      continue;
    }

    let Some(url) = &pack_mod.download.url else {
      continue;
    };

    match resolve_local(pack_mod, url, &cache_dir, no_download).await {
      Some(project) => projects.push(Project::Local(project)),
      None => {
        if !no_download {
          log::warn!(
            "Could not resolve custom mod \"{}\" from jar at {url}",
            pack_mod.name
          );
        }
      }
    }
  }

  projects
}
