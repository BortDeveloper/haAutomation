# Security Policy

## Supported Versions

This project is in active development. Only the `main` branch is
currently supported; there are no tagged releases yet.

| Version | Supported          | Status          |
| ------- | ------------------ | --------------- |
| `main`  | Yes (best effort)  | **alpha**       |
| Other   | No                 | — no releases — |

Because the project is alpha-state, expect breaking changes between
commits. Security fixes will land on `main` and will not be
back-ported.

## Reporting a Vulnerability

If you discover a security issue in this repository, please report it
privately. Do **not** open a public GitHub issue.

**Preferred channel**: open a private
[GitHub Security Advisory](https://github.com/BortDeveloper/haAutomation/security/advisories/new)
for this repository.

**Alternative**: email
`31363351+BortDeveloper@users.noreply.github.com` with a short
description, reproduction steps, and (if applicable) a proof-of-concept.

Please include:

- Affected file(s) / commit SHA
- A clear description of the issue and its security impact
- Steps to reproduce, or a minimal proof-of-concept
- Your assessment of severity (CVSS optional but appreciated)
- Whether you would like public credit after disclosure

## Disclosure Timeline

This project follows a **90-day coordinated disclosure** policy:

| Day  | Step                                                          |
| ---- | ------------------------------------------------------------- |
| 0    | Report received, acknowledged within 7 days                   |
| 7-60 | Investigation, fix development, fix review                    |
| 60-90| Fix merged to `main`, advisory drafted                        |
| 90   | Public disclosure (GitHub Security Advisory + CVE if eligible) |

The window may be shortened by mutual agreement (e.g., if the bug is
already public) or extended for complex coordinated fixes — but never
unilaterally extended without dialogue with the reporter.

## Public Acknowledgments

Reporters who follow this policy and request credit will be named in
the published advisory. If you prefer to remain anonymous, that is
honored without question.

## What is in scope

- The Rust inventory backend in `home-inventory/`
- The CI workflows in `.github/workflows/`
- The Docker setup in `home-inventory/docker/`
- The edge-secrets tooling in `edge-secrets/`
- Third-party dependency hygiene: see
  [THIRD-PARTY-NOTICES.md](./THIRD-PARTY-NOTICES.md) for the current
  inventory (auto-generated from `Cargo.lock`, CI-enforced via the
  `license-notices` job) and `home-inventory/deny.toml` for the accepted
  SPDX allowlist enforced by `cargo deny`.

## What is NOT in scope

- Third-party services referenced by the docs (Home Assistant,
  Node-RED, Homematic, Authentik, Caddy) — please report those
  upstream.
- Findings that require a compromised local network or already-leaked
  credentials.
- Reports based purely on dependency-tree analysis without an
  exploitable code path; please file these as regular issues with the
  `security/triage` label.

## Note

This is an alpha project run by a single maintainer in their spare
time. Response times are best-effort, not contractual. Thank you for
your patience and for reporting responsibly.
