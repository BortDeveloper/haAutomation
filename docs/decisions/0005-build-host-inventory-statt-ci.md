# ADR-0005: Dedizierter Build-Host `inventory` statt CI (V1)

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert)
- **Bezug:** Architekt-Audit R6, Finding 10 (kein CI)

## Kontext

Der `inventory`-Crate hängt über `ureq`→`ring` und `rusqlite`→`libsqlite3-sys`
an C-Code (siehe [ADR-0001](0001-rust-synchroner-minimal-stack.md)). Der
mingw-`gcc` der Windows-Workstation ist defekt — `cargo build`/`cargo test`
schlagen dort **immer** fehl, auch für das `authgate`-Binary allein, weil der
gesamte Dependency-Graph gezogen wird. Eine CI-Pipeline existiert in V1 nicht.

## Entscheidung

- Build und Tests laufen auf einem **dedizierten Linux-Host `inventory`** im
  Tailscale-Tailnet (Debian 12, `cargo`/`rustc` via `rustup`).
- Auf der Workstation läuft nur das Dependency-Resolve (`Cargo.lock`).
- Eine echte CI-Pipeline (`gitleaks`-Hook → CI, `clippy`, `hadolint`,
  `cargo test`) ist als Roadmap-Schritt **S19 / Phase 2** vorgemerkt. Der
  pre-commit-`gitleaks`-Hook (Closing-Brief §2.4) ist die Stop-Gap-Variante am
  Entwicklerplatz, **nicht** der Zielzustand.

## Folgen

**Positiv**

- Builds und die 52 cargo-Tests sind reproduzierbar lauffähig.

**Negativ / Kosten**

- Kein automatisches Test-Gate — Tests werden manuell angestoßen.
- Migration auf CI ist in Phase 2 verbindlich nachzuholen (S19).
