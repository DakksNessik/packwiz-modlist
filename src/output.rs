use log::info;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWrite, AsyncWriteExt};

use crate::args::SortingMode;
use crate::cache::Cache;
use crate::data::{get_data, get_projects};
use crate::object::{Data, Project};
use crate::{Args, GlobalError, GlobalResult, ValidationError};

pub fn display_project(index: usize, format: &str, project: &Project) -> String {
  format
    .replace("{INDEX}", &index.to_string())
    .replace("{TITLE}", &project.title())
    .replace("{NAME}", &project.title())
    .replace("{DESCRIPTION}", &project.description())
    .replace("{SUMMARY}", &project.description())
    .replace("{URL}", &project.url())
    .replace("{ID}", &project.id())
    .replace("{SLUG}", &project.slug())
    .replace("\\n", "\n")
}

pub async fn generate(cache: &mut Cache, args: &Args) -> GlobalResult<Data> {
  let (pack, mods) = get_data(args)?;
  let projects = get_projects(cache, &mods, args).await?;

  Ok(Data {
    pack,
    mods,
    projects,
  })
}

pub async fn write_projects<W>(args: &Args, data: &Data, writer: &mut W) -> GlobalResult<()>
where
  W: AsyncWrite + Unpin,
{
  let mut projects = data.projects.clone();

  if let Some(mode) = args.sort_by {
    projects.sort_by(|a, b| match mode {
      SortingMode::Name | SortingMode::Title => {
        a.title().to_lowercase().cmp(&b.title().to_lowercase())
      }
      SortingMode::Slug => a.slug().cmp(&b.slug()),
      SortingMode::Id => a.id().cmp(&b.id()),
    });
  }

  if args.reverse {
    projects.reverse();
  }

  for (index, project) in projects.iter().enumerate() {
    let display = display_project(index, &args.format, project);

    info!("{display}");

    writer.write_all(&display.into_bytes()).await?;
  }

  Ok(())
}

pub async fn write(args: &Args, data: &Data) -> GlobalResult<()> {
  match &args.output {
    Some(path) => {
      let path = if args.output_custom {
        path.clone()
      } else {
        args.path.join(path)
      };

      if path.exists() && !args.force {
        return Err(GlobalError::Validation(
          ValidationError::OutputAlreadyExits(path),
        ));
      }

      let mut file = File::create(path).await?;

      write_projects(args, data, &mut file).await?;
    }
    None => {
      write_projects(args, data, &mut stdout()).await?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::object::{ModrinthProject, Project};

  fn modrinth_project() -> Project {
    Project::Modrinth(ModrinthProject {
      id: "abc123".into(),
      slug: "sodium".into(),
      team: "team_abc".into(),
      team_members: Vec::new(),
      icon_url: None,
      source_url: None,
      title: "Sodium".into(),
      description: "A modern rendering engine".into(),
    })
  }

  #[test]
  fn display_replaces_all_placeholders() {
    let project = modrinth_project();
    let out = display_project(
      0,
      "- [{INDEX}] {NAME} - {DESCRIPTION} ({URL}) [id={ID} slug={SLUG}]\n",
      &project,
    );

    assert_eq!(
      out,
      "- [0] Sodium - A modern rendering engine (https://modrinth.com/mod/abc123) [id=abc123 slug=sodium]\n"
    );
  }

  #[test]
  fn display_uses_one_based_index() {
    let project = modrinth_project();
    assert_eq!(display_project(0, "{INDEX}", &project), "0");
    assert_eq!(display_project(1, "{INDEX}", &project), "1");
    assert_eq!(display_project(41, "{INDEX}", &project), "41");
  }

  #[test]
  fn display_title_and_name_are_equivalent() {
    let project = modrinth_project();
    assert_eq!(
      display_project(0, "{NAME}|{TITLE}", &project),
      "Sodium|Sodium"
    );
  }

  #[test]
  fn display_description_and_summary_are_equivalent() {
    let project = modrinth_project();
    assert_eq!(
      display_project(0, "{DESCRIPTION}|{SUMMARY}", &project),
      "A modern rendering engine|A modern rendering engine"
    );
  }

  #[test]
  fn display_untouched_placeholders_remain() {
    let project = modrinth_project();
    assert_eq!(
      display_project(0, "{NAME} {UNKNOWN}", &project),
      "Sodium {UNKNOWN}"
    );
  }

  #[test]
  fn display_expands_literal_backslash_n() {
    let project = modrinth_project();
    assert_eq!(
      display_project(0, "{NAME}\\n{URL}", &project),
      "Sodium\nhttps://modrinth.com/mod/abc123"
    );
  }
}
