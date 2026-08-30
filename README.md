[![crates.io](https://img.shields.io/crates/v/packwiz-modlist.svg)](https://crates.io/crates/packwiz-modlist)
[![license](https://img.shields.io/github/license/Ricky12Awesome/packwiz-modlist)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blueviolet)](https://www.rust-lang.org/)
![Platforms](https://img.shields.io/badge/platform-linux%20%7C%20windows-lightgrey)

# Packwiz ModList

> A CLI that turns a [packwiz](https://packwiz.infra.link/) Minecraft modpack into a human-readable markdown (or JSON) modlist.

`packwizml` reads a packwiz project (its `pack.toml` and the mod metadata files in `mods/`), resolves each mod against the **Modrinth** and **CurseForge** APIs, and renders a modlist you can drop into your modpack's README — or export as JSON for scripts.

---

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Options](#options)
- [Placeholders](#placeholders)
- [Local / custom mods & the metadata cache](#local--custom-mods--the-metadata-cache)
- [Sorting](#sorting)
- [CurseForge API key](#curseforge-api-key)
- [Development & Build](#development--build)
- [Roadmap](#roadmap)
- [License](#license)

---

## Install

Build from source with Cargo (a Rust toolchain is required):

```shell
cargo install packwiz-modlist
```

You can also grab a prebuilt binary from the [Releases](https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/releases) page (Linux musl, Linux glibc, and Windows are built automatically by Gitea Actions).

## Usage

Running `packwizml` with no arguments prints the modlist in the default format to stdout. Redirect the output with `-o` or `> filename`.

```shell
# Print a modlist for the current directory
packwizml

# Write it to a file
packwizml -o modlist.md

# Point at a different packwiz project
packwizml --path /path/to/pack -o modlist.md
```

## Options

```sh
# Displays help
packwizml --help # short: -h

# Prints about this program
packwizml --about

# Sets a project path
# other path options are relative to this by default
# to disable this, add '-O' for output and '-M' for mods
packwizml --path ./ # short: -p

# Sets an output directory
# to disable being relative to '--path' add '-O'
packwizml --output modlist.md # short: -o

# Sets a mods directory
# to disable being relative to '--path' add '-M'
packwizml --mods ./mods # short: -m

# Overwrite output file if it exists
packwizml --force # short: -F

# Prints out all data as json so it can be used in scripts
packwizml --json

# Sets the logging level
# possible values: Off, Error, Warn, Info, Debug, Trace
# default: Warn
packwizml --log-level=Off # short: -v

# Sets the color mode
# possible values: Auto, Always, Never
# default: Auto
packwizml --color-mode=Auto # short: -c

# Sets a custom format
# default: `- [{NAME}]({URL}) - {DESCRIPTION}\n`
packwizml --format "- [{NAME}]({URL}) - {DESCRIPTION}\n" # short: -f

# Sets how it should sort
# possible values: Name, Title, Slug, Id, None
packwizml --sort-by Name # short: -s

# Sets if sorting should be reverse
packwizml --reverse # short: -r

# Sets the cache file
# default: .packwiz-modlist.cache.json
packwizml --cache .packwiz-modlist.cache.json

# Sets the directory where downloaded mod metadata is cached
# default: .cache (relative to --path unless '-D')
packwizml --cache-dir .cache # short: -D

# Skip downloading jars; only use cached/extracted metadata
packwizml --no-download # short: -n
```

> **Cache & ignore files:** on every run `packwizml` makes sure `.cache` and the modlist cache
> file (`.packwiz-modlist.cache.json`) are ignored by both `.gitignore` and `.packwizignore` at
> the pack root. It creates the files if missing and appends entries idempotently, so regenerable
> artifacts stay out of git and out of packwiz's index.

### Placeholders

These placeholders are available in the `--format` string:

| Placeholder                  | Description                      |
|:-----------------------------|:---------------------------------|
| `{INDEX}`                    | Gets project index in the list   |
| `{NAME}`, `{TITLE}`          | Gets project name/title          |
| `{DESCRIPTION}`, `{SUMMARY}` | Gets project description/summary |
| `{URL}`                      | Gets project URL                 |
| `{SLUG}`                     | Gets project slug                |
| `{ID}`                       | Gets project id                  |

### Local / custom mods & the metadata cache

`packwizml` can resolve mods that aren't (or can't be) looked up through the Modrinth / CurseForge
APIs by reading the mod's own `.jar`:

* **Custom mods** — published on neither store and carrying a direct `[download].url` (for example
  a bundle jar hosted on Gitea or GitHub) — are resolved from their jar. Mods with no `[update]`
  section are now supported instead of crashing with `missing field 'update'`.
* **CurseForge fallback** — when the CurseForge API is unavailable or no key is set, `packwizml`
  reconstructs the CDN URL from the mod's `file-id` + `filename`
  (`https://edge.forgecdn.net/files/{fid/1000}/{fid%1000}/{filename}`), downloads the jar, and
  extracts real metadata. A bare placeholder is only used as a last resort.

Metadata is extracted from `META-INF/neoforge.mods.toml`, `META-INF/mods.toml`, or
`fabric.mod.json` (whichever is present).

To limit bandwidth and API abuse, the extracted metadata is cached as `<download-hash>.toml` in
`--cache-dir` (default `.cache/`), and the downloaded `.jar` is deleted after extraction. Re-runs
with the same file hash reuse the cache with **zero network traffic**; updating a mod changes its
hash, which is a cache miss and triggers a fresh download. Use `--no-download` / `-n` to run purely
from the cache.

### Sorting

| Type                         | Description                      |
|:-----------------------------|:---------------------------------|
| `Name`, `Title`              | Sorts by project name            |
| `Slug`                       | Sorts by project slug            |
| `Id`                         | Sorts by project id              |
| `None`                       | Undetermined                     |

## CurseForge API key

The CurseForge API requires an API key, which is read at **runtime** (it is **not** needed at build time):

* Set `CF_API_KEY` in the environment, e.g. `export CF_API_KEY=...`
* or place it in a `.env` file in the working directory (`CF_API_KEY=...`), which is loaded automatically via [dotenvy](https://crates.io/crates/dotenvy).

The key is only required when the pack actually contains CurseForge mods; packs with only Modrinth mods work without a key.

> **Fallback behavior:** If the CurseForge API is unavailable (for example its edge CDN returns empty-body `403` responses) or no key is set, `packwizml` no longer aborts the run. It reconstructs each CurseForge mod's CDN URL and extracts real metadata from the jar (see [Local / custom mods & the metadata cache](#local--custom-mods--the-metadata-cache)). A placeholder entry — using the mod's name and a link to its numeric project ID — is used only as a last resort if the download or extraction fails. The command exits `0` and prints a warning to stdout unless logging is turned off or silenced (log level below `Warn`). Placeholders are not written to the cache, so a later successful run replaces them with real data.

> **Note:** the `.env` file is git-ignored so your key is never committed to the repository.

## Development & Build

The project is written in Rust (edition 2021). To build locally:

```shell
cargo build --release
```

The `tagged_release.yml` workflow on Gitea Actions builds and releases three binaries on every `v*` tag:

| Platform      | Toolchain | Notes                                     |
|:--------------|:----------|:------------------------------------------|
| Linux (musl)  | Alpine    | Fully static binary                       |
| Linux (glibc) | `rust:1-bookworm` | Built in a Debian container via Docker |
| Windows       | MSVC      | Built on a Windows runner with VS Build Tools |

## Roadmap

- [x] Sorting
- [x] Use CurseForge official API
- [x] Caching (avoid re-fetching projects on the same version)
- [ ] Automated Tests
- [ ] Packaging outside of cargo
- [ ] Packaging with completions
- [ ] Templates (kinda like preset-format)

## License

Distributed under the [Apache-2.0](LICENSE) license.
