# Architecture Decision Records (ADR)

Dieses Verzeichnis enthält die **repo-internen Architekturentscheidungen** für
`ha-automation`. Sie wurden gemäß Architekt-Audit-Finding **R6**
(`docs/cockpit-audits/2026-05-16-architect.md`) und Closing-Brief §4
retroaktiv dokumentiert — die Entscheidungen waren im Code/Repo bereits
getroffen, aber nicht als ADR festgehalten.

## Abgrenzung: repo-lokal vs. Cockpit

| Ebene | Ort | Zitierweise |
|---|---|---|
| **Repo-lokal** (dieses Verzeichnis) | `docs/decisions/NNNN-*.md` | „ADR-0001" |
| **Cockpit-weit** (normativ, phasenübergreifend) | `stack-master/shared/architecture-decisions/` | „Cockpit-ADR-0001" |

Die Nummernkreise sind **unabhängig**. Ein „ADR-0004" hier ist nicht
„Cockpit-ADR-0004". Bei Konflikt gilt die Cockpit-Ebene (Single Source of
Truth für phasenübergreifende Architektur).

## Format

Schlank an [MADR](https://adr.github.io/madr/) angelehnt: **Status · Kontext ·
Entscheidung · Folgen**. Ein ADR wird nicht rückwirkend umgeschrieben — eine
geänderte Entscheidung bekommt ein neues ADR, das das alte als `superseded`
markiert.

Status-Werte: `proposed` · `accepted` · `superseded by ADR-NNNN` · `deprecated`.

## Index

| ADR | Titel | Status |
|---|---|---|
| [0001](0001-rust-synchroner-minimal-stack.md) | Rust + synchroner Minimal-Stack (kein tokio) | accepted |
| [0002](0002-sqlite-cache-yaml-source-of-truth.md) | SQLite als Cache, YAML als Source of Truth | accepted |
| [0003](0003-vpn-provider-abstraktion-service-vpn.md) | VPN-Provider-Abstraktion über `service:vpn` | accepted |
| [0004](0004-authgate-uebergangs-sso.md) | `authgate` als Übergangs-SSO, Zielzustand Authentik | accepted |
| [0005](0005-build-host-inventory-statt-ci.md) | Dedizierter Build-Host `inventory` statt CI (V1) | accepted |
| [0006](0006-image-build-on-host-statt-registry.md) | Image-Build auf dem Zielhost statt Registry-Pull | accepted |
| [0007](0007-eu-ki-vo-nicht-anwendbar.md) | EU-KI-VO: nicht anwendbar (Negativ-ADR) | accepted |
| [0008](0008-eu-data-act-mapping.md) | EU Data Act: Anwendbarkeit + Artikel-Mapping | accepted |

## Offene / vertagte Punkte (kein ADR)

- **R8 — Metrics-Endpoint + JSON-Lines-Logs**: Phase-2-Vorlauf (Gate G3.5),
  Rust-Code-Änderung. Noch kein ADR, da die konkrete Umsetzung (Prometheus-
  Format, Log-Crate) erst mit der Implementierung entschieden wird.
- **Folge-ADR Backup-Target**: S3-Provider-Wahl für das Off-Site-Backup
  (Closing-Brief §2.3) — User-Entscheidung ausstehend, siehe [ADR-0008](0008-eu-data-act-mapping.md).
