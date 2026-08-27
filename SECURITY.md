# Security Policy

## Reporting a Vulnerability

Please **do not** file a public issue for security vulnerabilities. Instead, report privately so it can be addressed before disclosure.

**Preferred:** GitHub private vulnerability reporting (Settings → Security → *Report a vulnerability*) for this repository.

**Alternative:** Email the maintainer directly for this fork (Wes Sims).

For issues that affect the **upstream** project, report to [Ricky12Awesome/packwiz-modlist](https://github.com/Ricky12Awesome/packwiz-modlist) instead.

Please include, where possible:

- The affected version (`packwizml --version`)
- Steps to reproduce (ideally a minimal pack / command)
- The impact and any suggested fix

You should receive an acknowledgement within 3 business days, and we'll aim to provide a fix or a timeline within 7 business days.

## Supported Versions

Only the **latest release** is actively supported for security fixes. Older releases may receive fixes on a best-effort basis; please update to the newest release when possible.

## Security Context

This project is a **fork** of the upstream `packwiz-modlist` CLI. Security-relevant behavior to be aware of:

- **Runtime API keys:** `CF_API_KEY` is read at runtime from the environment or a `.env` file and is **never** committed. The `.env` file is git-ignored. Never paste a real key into issues, PRs, logs, or crash output.
- **Network access:** `packwizml` makes HTTPS requests to the Modrinth and CurseForge APIs (via rustls). It does not execute downloaded content.
- **Supply chain / CI:** All GitHub Actions are pinned to commit SHAs and the repository's Actions permission is restricted to a `selected` allowlist. Release binaries are built and smoke-tested by CI on every tag.

## Security best practices for maintainers / contributors

- Treat the contents of `pack.toml` / mod metadata as untrusted input — a malicious pack should never cause unsafe behavior.
- Do not introduce new runtime dependencies that require C toolchains without considering the musl/static-build impact.
- When adding or updating a GitHub Action, pin it by full commit SHA and add the SHA to the repo's `selected-actions` allowlist.

## Disclosures

We follow responsible disclosure: we ask that you give us time to fix and release before publicizing a vulnerability.
