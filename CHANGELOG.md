# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A `Validate` GitHub Actions workflow that runs on every PR and push to `main`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, plus release-binary smoke tests on Linux, macOS, and Windows (each built binary is executed with `--version`, `--help`, and `--about`, then uploaded as an artifact).
- Unit tests covering format placeholder expansion (`{INDEX}`, `{NAME}`, `{DESCRIPTION}`, `{URL}`, `{ID}`, `{SLUG}`, literal `\n`) and the `Project` accessors / Modrinth / CurseForge URL construction.

### Changed

- The release pipeline now runs on standard GitHub-hosted runners (`ubuntu-latest`, `macos-latest`, `windows-latest`) instead of self-hosted Gitea runners, and adds a **macOS (Apple Silicon / aarch64)** build. Linux binaries are produced for both musl (fully static) and glibc.
- All GitHub Actions are pinned to full commit SHAs to protect against mutable-tag supply-chain attacks, and the repository's Actions permission is restricted to `selected` (GitHub-owned actions plus the pinned third-party actions).

### Fixed

- Pre-existing `cargo clippy` warnings (redundant closure, needless borrow, suspicious `open_options`) and normalized formatting so the lint gate passes.

## [1.5.4+hotfix.1] - 2026-08-26

### Added

- When the CurseForge API is unavailable (for example its edge CDN returns empty-body `403` responses) or no key is set, `packwizml` now falls back to **placeholder** entries for CurseForge mods instead of aborting the whole run, so a partial modlist/README can still be produced from the resolved Modrinth mods. The command exits `0`, and a warning is printed to stdout unless logging is turned off or silenced (log level below `Warn`). Placeholders are built from the local packwiz metadata and link to the numeric project ID (which CurseForge redirects to the real slug), and are not written to the cache so a later successful run replaces them with real data.

## [1.5.4] - 2026-08-26

### Changed

- `CF_API_KEY` is now read at **runtime** (via environment variable or a `.env` file using [dotenvy](https://crates.io/crates/dotenvy)) instead of at compile time. The crate now compiles without a CurseForge API key present.
- Refreshed all dependencies to their latest compatible versions (`cargo update`).
- Switched the HTTP client from `reqwest`'s default native-TLS/OpenSSL backend to **rustls**, removing the OpenSSL build dependency (and enabling a fully static musl binary).
- Rewrote the release pipeline as a Gitea Actions workflow that builds and publishes Linux (musl), Linux (glibc), and Windows (MSVC) binaries on every `v*` tag.

### Fixed

- The `packwizml` binary no longer fails to build when `CF_API_KEY` is unset — the key is only required at runtime, and only when a pack contains CurseForge mods.
- A missing CurseForge key now produces a clear runtime error instead of a compile-time failure.

### Security

- `.env` is now git-ignored and untracked so the CurseForge API key is never committed to the repository.

## [1.5.3] - 2022-07-08

> Note: the upstream repository never tagged `v1.5.3`; this entry reflects the state of the code between `v1.5.2` and this fork's `v1.5.4`.

### Changed

- Use `.env` and `dotenv-build` to supply the CurseForge API key at build time (upstream change, superseded by the runtime handling in `v1.5.4`).

## [1.5.2] - 2022-07-08

### Added

- Support for the official CurseForge API (requires a `CF_API_KEY`).

## [1.5.1] - 2022-06-24

### Changed

- Updated dependencies.

## [1.5.0] - 2022-06-24

### Changed

- Modrinth now uses the project id instead of the slug (cannot be done for CurseForge).

## [1.4.0] - 2022-06-05

### Added

- Project caching, to avoid re-fetching a project by URL when the version is unchanged.
- Updated shell completions.

### Changed

- Reworked how output is written.
- Removed the automatic trailing newline so users can fully control the output format.
- Added sorting (`Name`, `Title`, `Slug`, `Id`).
- Ran the code through `rustfmt`.

## [1.3.0] - 2022-06-03

### Added

- Auto-completion support (generated shell completions).

### Changed

- `ColorMode::from_str` no longer returns an error since clap validates values.

## [1.2.1] - 2022-04-15

### Fixed

- Release artifact file names now include the OS and architecture.

## [1.2.0] - 2022-04-15

### Added

- JSON output mode (`--json`).
- `--about` command.
- Color mode argument (`--color-mode`).

### Changed

- Reworked how verbosity works.

## [1.1.0] - 2022-04-15

### Added

- Added a workflow for automatic releases on tagged commits.

## [1.0.2] - 2022-04-14

### Changed

- Removed an unused dependency.

## [1.0.1] - 2022-04-10

### Fixed

- Fixed a type issue.

## [1.0.0] - 2022-04-10

### Added

- Initial release. Reads a packwiz project and generates a markdown modlist from Modrinth and CurseForge projects.

[Unreleased]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.4...main
[1.5.4+hotfix.1]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.4...v1.5.4+hotfix.1
[1.5.4]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.2...v1.5.4
[1.5.3]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.2...v1.5.3
[1.5.2]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.1...v1.5.2
[1.5.1]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.5.0...v1.5.1
[1.5.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.4.0...v1.5.0
[1.4.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.3.0...v1.4.0
[1.3.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.2.1...v1.3.0
[1.2.1]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.2.0...v1.2.1
[1.2.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.1.0...v1.2.0
[1.1.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.0.2...v1.1.0
[1.0.2]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.0.1...v1.0.2
[1.0.1]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/compare/v1.0.0...v1.0.1
[1.0.0]: https://gitea.crazygnome.net/wessims.jr/packwiz-modlist/releases/tag/v1.0.0
