# home-inventory — Home Automation Inventory & Refactor

> ## ⚠️ ALPHA / EXPERIMENTAL — NOT FOR PRODUCTION USE
>
> **Status (2026-05-19)**: This project is in active development. Build,
> tests, and CI are functional, but the codebase has not undergone
> independent security review, integration testing in a production
> environment, or long-term operational validation.
>
> **No warranty**: The author(s) provide this software "as is", without
> warranty of any kind, express or implied. There is no guarantee of
> functionality, fitness for any particular purpose, security, or
> stability. See [LICENSE](./LICENSE) for the full MIT terms.
>
> **No liability**: Use at your own risk. The author(s) accept no
> responsibility for any damages, data loss, security incidents, or
> other harm arising from the use of this software — direct, indirect,
> incidental, or consequential.
>
> **Recommendation**: This package is **not suitable for production
> deployment** in its current state. Use only for evaluation,
> development, or research purposes.
>
> Vulnerability reports: [SECURITY.md](./SECURITY.md).

> ## ⚠️ ALPHA / EXPERIMENTELL — KEINE PRODUKTIONSEIGNUNG
>
> **Stand (2026-05-19)**: Dieses Projekt befindet sich in aktiver
> Entwicklung. Build, Tests und CI sind funktionsfähig, aber der
> Code wurde nicht unabhängig sicherheitsgeprüft, nicht in einer
> Produktionsumgebung integrationsgetestet und nicht langfristig
> betriebsvalidiert.
>
> **Keine Gewähr**: Die Autor(inn)en stellen diese Software "wie
> besehen" zur Verfügung, ohne jegliche ausdrückliche oder
> stillschweigende Gewährleistung. Es besteht keine Zusicherung
> hinsichtlich Funktion, Eignung für einen bestimmten Zweck,
> Sicherheit oder Stabilität. Vollständige MIT-Lizenz: [LICENSE](./LICENSE).
>
> **Keine Haftung**: Nutzung auf eigene Gefahr. Die Autor(inn)en
> übernehmen keinerlei Verantwortung für Schäden, Datenverluste,
> Sicherheitsvorfälle oder sonstige Beeinträchtigungen, die aus
> der Nutzung dieser Software entstehen — weder direkt, indirekt,
> beiläufig noch in der Folge.
>
> **Empfehlung**: Dieses Paket ist im aktuellen Stand **nicht für
> einen produktiven Einsatz geeignet**. Verwende es ausschließlich
> für Evaluation, Entwicklung oder Forschungszwecke.
>
> Vulnerability-Reports: [SECURITY.md](./SECURITY.md).

**Third-party software**: This project depends on open source
components. See [THIRD-PARTY-NOTICES.md](./THIRD-PARTY-NOTICES.md) for
the complete, auto-generated list of dependencies, versions, and
sources. CI enforces (a) byte-for-byte sync with `home-inventory/Cargo.lock`
on every push (`.github/workflows/security.yml`, job `license-notices`)
and (b) a permissive-license allowlist via `cargo deny`
([`home-inventory/deny.toml`](./home-inventory/deny.toml)). Copyleft licenses
are rejected.

Single Source of Truth for a personal home-automation setup. Captures,
documents and refactors logic spread across Home Assistant, Node-RED
and Homematic. Provides its own Rust-based inventory backend that runs
on a public VPS and inspects the home network through a VPN tunnel.

**Status:** Phase 1 + 2 + 3 complete, 52 cargo tests green. VPS deploy
(VPN sidecar + Caddy + Authentik forward_auth) is up next. Details:
[docs/roadmap.md](docs/roadmap.md).

## Where to start

Three audiences, three entry points:

| Audience | Interest | Starting question | Primary document |
|---|---|---|---|
| Technically interested | Overview | What does this do, and why? | This README + [Requirements](docs/requirements.md) |
| Operators | Running it | Where do I press what? | [Runbooks](docs/runbooks/) — install, VPS, edge-secret restore |
| Developers | Extending it | How is it built, how do I extend it? | [Architecture](docs/architecture.md) + [ADRs](docs/decisions/README.md) |

### I want to …

| I want to … | Document |
|---|---|
| set up the PoC on my home network | [Getting started](docs/getting-started.md) |
| build and run it on a Raspberry Pi | [Raspberry Pi install](docs/runbooks/raspberry-pi-install.md) |
| bootstrap the public VPS | [VPS setup](docs/vps-setup.md) |
| restore the edge secrets after key loss | [Edge-secret backup runbook](docs/runbooks/edge-secret-backup.md) |
| generate operator keys (ADR-0004) | [Operator checklist](docs/runbooks/operator-checklist-adr-0004.md) |
| understand trust boundaries and data flow | [Architecture](docs/architecture.md) |
| look up a design decision | [ADR index](docs/decisions/README.md) |
| browse all documentation | [docs/](docs/README.md) |

## Components

| Area | Role | Where in the repo |
|---|---|---|
| Home Assistant (HAOS) | UI, state registry, simple automations | `homeassistant/` (planned) |
| Node-RED (HA addon) | complex flows, timers, notifications | `nodered/` (planned) |
| Homematic CCU (RaspberryMatic) | direct device links, latency-critical devices | `homematic/` (planned) |
| Philips Hue (multi-bridge) | dimmable lights + sensors, via v1 REST | covered by inventory |
| Shellys (20+, mDNS) | switches / rollers / sensors, Gen1+Gen2 HTTP | covered by inventory |
| Zigbee2MQTT (HA addon) | Zigbee devices over MQTT (phase-5 sync) | `nodered/` (planned) |
| Inventory backend (Rust) | collects devices/firmware/software, web UI | `home-inventory/` |
| Docs & mapping | as-is state, ownership, ADRs | `docs/` |

## Sync sources (active)

| Source | CLI | Provides | Firmware tracking |
|---|---|---|---|
| Home Assistant | `home-inventory sync ha --url --token` | devices via `/api/states` (14 domains filtered) | no (HA API) |
| Homematic CCU | `home-inventory sync ccu --url` | devices via `xmlapi/devicelist.cgi` | yes, diff-based |
| Philips Hue | `home-inventory sync hue --config <yaml>` | lights + sensors, multiple bridges | yes, diff-based |
| Shelly | `home-inventory sync shelly [--ip ...] [--discover-seconds N]` | Gen1+Gen2 via HTTP, mDNS auto-discovery | yes, diff-based |

## Documentation

A full, categorized index lives in [docs/README.md](docs/README.md). Most-used
entries:

**Setup and first run**

- [Getting started](docs/getting-started.md) — first-run PoC on a home network (security-audited walkthrough)
- [Raspberry Pi install](docs/runbooks/raspberry-pi-install.md) — prepare a Pi, build the suite, run the HA sync
- [VPS setup](docs/vps-setup.md) — server bootstrap, SSH/keys, ops, DR

**Operating (Day-2)**

- [Runbooks](docs/runbooks/) — edge-secret backup/restore, coordinator replacement, Node-RED flow update
- [Edge-secret backup](docs/runbooks/edge-secret-backup.md) — sops/age escrow, restore drill (Cockpit ADR-0004)
- [Operator checklist ADR-0004](docs/runbooks/operator-checklist-adr-0004.md) — key generation steps

**Architecture and decisions**

- [Requirements](docs/requirements.md) — what is being built and why
- [Architecture](docs/architecture.md) — components, data flow, trust boundaries
- [Roadmap](docs/roadmap.md) — step plan S1–S14
- [ADRs](docs/decisions/README.md) — design decisions with rationale

**Code**

- [Inventory backend](home-inventory/) — Rust app (frozen since 2026-06-14; successor: `home-inventory-c`)
- [Docker / VPN](home-inventory/docker/README.md) — sidecar setup, provider switch
- [Secrets](home-inventory/secrets/) — sops+age layout (inventory ops secrets)

## Conventions

- **GitOps:** everything except the age private key lives in the repo
- **PRs with test gate:** each roadmap step = one PR, with a reproducible test
- **Language:** code & identifiers in English, docs in English
- **License:** MIT (see [LICENSE](LICENSE)); third-party dependencies
  are listed in [THIRD-PARTY-NOTICES.md](./THIRD-PARTY-NOTICES.md)
  (auto-generated, CI-enforced)

## Anonymization note

This repository is shared publicly. Identifying values (hostnames, IPs,
emails, location/room names) have been replaced with placeholders:

| Placeholder | Meaning |
|---|---|
| `<vps-host>` / `vps.example.org` | the public VPS hostname |
| `example.org` | any private domain |
| `<ha-host>` / `homeassistant.example.local` | the Home Assistant instance |
| `10.0.0.0/24` | the home subnet (RFC 1918 example) |
| `<your-email>` / `<your-name>` | maintainer identity in LICENSE |
| `room_a`, `room_b`, `hallway`, `outdoor_a` | generic room labels in fixtures |

Map these to your real values in a local config that is not committed.
