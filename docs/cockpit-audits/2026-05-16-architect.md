# Architect Audit – ha-automation – 2026-05-16

## Scope

- Projekt: `projects/ha-automation/` (Commit `52095ed feat(inventory): Paket A — vps-stack-tauglich`)
- Branch: `main`
- Phase: 1 (AUDIT), Iteration 0
- Typ laut `cockpit.yaml`: `edge`, Tech-Stack `[home-assistant, mqtt, zigbee]`
- Geprüft durch: `architect`
- Geprüft: `README.md`, `docs/{architecture,requirements,roadmap,vps-setup}.md`,
  `docs/memory/*`, `inventory/Cargo.toml`, `inventory/src/{main,auth,db,git_publish,http,secrets,views}.rs`,
  `inventory/src/bin/authgate.rs`, `inventory/src/sync/{ha,ccu,hue,shelly}.rs`,
  `inventory/migrations/0001_init.sql`, `inventory/docker/{Dockerfile,Caddyfile,docker-compose*.yml,justfile,README.md}`,
  `inventory/secrets/{.sops.yaml,*.example,vpn.wireguard/}`, `inventory/.dockerignore`,
  `.gitignore`, `scripts/trivy-scan.sh`.

## Findings

### Bestand (IST)

1. **Architekturbild abweichend vom Cockpit-Eintrag.** `cockpit.yaml` deklariert ha-automation als
   Edge-Projekt mit Stack `[home-assistant, mqtt, zigbee]`. Das tatsächliche Repo enthält
   keine Home-Assistant-Konfiguration (`configuration.yaml`, `automations.yaml`,
   `scripts.yaml`, `secrets.yaml`, Custom Components, MQTT-/Zigbee-Setup), sondern
   ist ein **eigenes Inventarisierungs-Backend in Rust** (`inventory`-Crate, 5 Module +
   4 Sync-Quellen + `authgate`-Sidecar), das Geräte aus HA, Homematic CCU, Hue,
   Shelly über VPN inspiziert. HA/MQTT/Zigbee2MQTT sind im Repo **Beobachtungsziele**,
   nicht Bestandteil des Codes.

2. **Drei Deployment-Modi, ein Image.** `docker/docker-compose.yml` (Standalone:
   Caddy + `authgate` + austauschbarer VPN-Sidecar) und `docker/docker-compose.vps.yml`
   (Traefik + Authentik + host-VPN, ohne Caddy/authgate) teilen sich dasselbe Image
   und denselben Auth-Header-Vertrag `X-Authentik-Username`.

3. **Phase 1+2+3 laut Roadmap abgeschlossen** (52 cargo-Tests grün, Mock-basiert).
   Phase 4 (S13 Compose-VPN-Stack, S13a/b/c Overlays, S13d `authgate`, S14
   Authentik) steht aus. Live-Smoke gegen echte Heim-Systeme wurde bewusst gebündelt
   vertagt und ist noch nicht ausgeführt.

4. **Komponenten und SPoF.** Das System hat einen Single-Point-of-Failure auf zwei
   Ebenen: (a) der VPS-VPS (single Host, kein HA), (b) der age-Privatkey
   (`/etc/inventory/age.key`, „kein Backup auf demselben Host", Empfehlung
   Pubkey-Backup in Passwortmanager). Die Hardware-Annahme „ein VPS reicht" passt
   zum target-state-Nicht-Ziel „HA mit RPO=0", ist aber im Projekt-Repo nicht
   explizit als ADR begründet.

5. **VPN-Provider als drei austauschbare Compose-Overlays** (Tailscale, NetBird,
   WireGuard). Vertrag: ein Service heißt `vpn`, App joint
   `network_mode: service:vpn`. Sauberer Adapter-Pattern, App selbst kennt kein VPN.

6. **Secrets-Layer „sops + age".** `inventory/secrets/.sops.yaml` enthält noch zwei
   `# TODO`-Einträge ohne VPS-Host-Pubkey. Es liegen ausschließlich `.example`-
   Files im Repo, keine `.enc`-Files. Konsequenz: ein blanker Klon des Repos kann
   den Stack **nicht** ohne Out-of-Band-Schritt (Pubkey eintragen, Secrets neu
   verschlüsseln) hochfahren – das ist ein Hindernis für reproduzierbares Deployment.

7. **`:latest`-Tags in allen drei VPN-Overlays und Caddy** (`tailscale/tailscale:latest`,
   `netbirdio/netbird:latest`, `linuxserver/wireguard:latest`,
   `ghcr.io/getsops/sops:latest`, `caddy:2-alpine`). VPS-Compose pinnt
   demgegenüber per Digest (`INVENTORY_IMAGE=ghcr.io/.../inventory@sha256:<digest>`).
   Inkonsistenz zwischen Standalone- und VPS-Mode.

8. **Authgate-Sidecar als Übergangs-SSO.** PBKDF2-HMAC-SHA256 mit 600.000 Runden
   (OWASP-Empfehlung 2023), HMAC-SHA256 signiertes Session-Cookie, konstantzeitiger
   Vergleich, Dummy-PBKDF2 gegen User-Enumeration, Open-Redirect-Schutz, fail-closed
   bei leerer User-Liste. Auf Output-Ebene sauber. Architektonisch ist es aber ein
   **zweites Identity-Subsystem** parallel zum Cockpit-Ziel „Single Source of Identity:
   Keycloak" (`shared/target-state.md` § Architekturprinzipien #1). Der Wechsel ist
   per Caddyfile-Einzeiler vorgesehen, aber konzeptionell offen, welches Cockpit-IDP
   übernimmt: Keycloak (laut `cockpit.yaml.shared_services.sso.target_solution`) oder
   Authentik (laut Repo-Sprache und VPS-Setup).

9. **Container-Hardening unterschiedlich tief.** `docker-compose.vps.yml` setzt
   `read_only: true`, `cap_drop: [ALL]`, `no-new-privileges:true`, tmpfs, mem/cpu/
   pids-Limits, Healthcheck aus Image. `docker-compose.yml` (Standalone) setzt keines
   dieser Härtungs-Items für den `inventory`-Container; dafür hat das Image
   non-root-User (`USER inventory`), HEALTHCHECK und tini als PID 1.

10. **Kein CI im Repo, kein Linter-Wiring.** Keine GitHub-Actions, kein hadolint,
    yamllint, markdownlint, kein clippy-CI. Trivy-Scan existiert als manuelles
    Shell-Skript (`scripts/trivy-scan.sh`). `cargo test` und `docker build` werden
    laut Memory auf dem dedizierten `inventory`-Tailscale-Host gemacht, nicht
    automatisiert.

11. **KI/ML-Komponente: nicht vorhanden.** Das Projekt liest, schreibt und stellt
    Geräte-Inventardaten dar. Keine Inferenz, kein Modell, keine automatisierte
    Entscheidung im Sinne EU KI-VO Art. 3 Abs. 1. Roadmap-Phase 5 sieht „Migration
    bestehender Automationen pro Domäne" vor, aber Automationen wären regel-basiert
    (HA/Node-RED), nicht KI. → KIC-Konvergenz K1–K6 / A1–A3 ist heute **nicht
    anwendbar**; muss in einem App-ADR mit Artikelangabe explizit so dokumentiert
    werden (verlangt durch `shared/standards/eu-ai-act.md` § Audit-Hinweis).

12. **EU Data Act ist anwendbar.** Das System sammelt IoT-Geräte- und Firmware-Daten
    aus dem Heimnetz (HA, CCU, Hue, Shelly, später Zigbee2MQTT/Node-RED).
    Datenverordnung 2023/2854 Kap. II (Art. 3–7) regelt B2C/B2B-Zugang zu Daten
    vernetzter Produkte; Kap. VI (Art. 23–31) regelt Cloud-Switching, relevant für
    Backup-Targets und das spätere zentrale Backup (Restic→S3, laut
    `shared/target-state.md`). Beides ist im Repo bisher nicht referenziert.

13. **Dokumentation hochwertig und konsistent.** README.md, requirements.md,
    architecture.md, roadmap.md, vps-setup.md greifen sauber ineinander, der
    Lese- und Userpfad sind als ASCII-Diagramm gezeichnet, Ownership-Regeln (CCU
    vs. HA vs. Node-RED) sind explizit. Memory-Snapshots unter `docs/memory/`
    konservieren Projektkontext und Präferenzen. Sprache: Code englisch, Doku
    deutsch (konsistent).

### Abgleich gegen `shared/target-state.md`

| Prinzip | Soll | Ist im Projekt | Gap |
|---|---|---|---|
| Single Source of Identity (Keycloak) | Keycloak OIDC | `authgate` + Authentik geplant | offen, ADR nötig |
| TLS überall | TLS 1.2+ | Caddy + Let's Encrypt (Standalone), Traefik + LE (VPS) | OK |
| Secrets nie im Klartext | Vault/SealedSecret/sops | sops+age | OK, aber `.sops.yaml`-TODO blockiert Reproduzierbarkeit |
| Idempotenz | jede Aktion wiederholbar | Migrations idempotent, Upsert deterministisch, YAML-Export sortiert | OK |
| Beobachtbarkeit | Monitoring/Logs | Strukturierte stdout-Logs, kein Metrics-Endpoint, kein Healthcheck-Aggregat | Lücke gegenüber Soll-Zustand (Prometheus + Grafana, cockpit.yaml) |
| Doku ist Code | Repo, nicht Wiki | erfüllt | OK |
| ADR-Pflicht | für nicht-triviale Entscheidungen | aktuell **kein einziges Repo-internes ADR** (nur cockpit-weites ADR-0001) trotz vielzähliger Architekturentscheidungen | Lücke |

### Beobachtete Architektur-Entscheidungen ohne ADR

- Rust + tiny_http + ureq + rusqlite statt einer etablierten Stack-Kombination (NFR-2
  erwähnt die Entscheidung, aber kein ADR).
- SQLite + parallele YAML-Snapshots im Repo als duale Datenhaltung (FR-2: „YAML ist
  source of truth, SQLite Cache").
- VPN-Provider-Wahl Tailscale initial; Switching-Vertrag über `service:vpn`-Namen.
- `authgate` als Übergangs-SSO statt sofortige Keycloak-Integration.
- Build-Host „inventory" via Tailscale statt CI (Workstation kann den Crate nicht bauen).
- Image-Build auf dem Zielhost statt Registry-Push.

## Risks

- **[HIGH]** **Stack-Drift zwischen `cockpit.yaml` und Repo-Realität.** Der Eintrag
  „edge, tech_stack [home-assistant, mqtt, zigbee]" beschreibt Beobachtungsziele,
  nicht das Repo. Folge: spätere Audits (Security, SRE) könnten Linter und Quality
  Gates für die falsche Technologie planen (z. B. ansible-lint statt cargo clippy
  und hadolint). Bezug: **Dokumentation**, `MISSION.md` § „Doku ist Code".
  Quelle: Cockpit-Konvention `config/cockpit.yaml`, abgeglichen mit `README.md` § Komponenten.

- **[HIGH]** **Doppeltes IDP-Konzept ohne ADR-Klärung.** Repo plant Authentik
  (S14, `forward_auth`, Caddyfile), Cockpit plant Keycloak (`cockpit.yaml.shared_services.sso` +
  `shared/target-state.md`). Solange das nicht durch ein ADR aufgelöst ist,
  besteht das Risiko, dass das Projekt in Phase 3 ein zweites IDP betreibt oder
  einen vermeidbaren Refactor durchläuft. Bezug: **Sicherheit + Komplexität**.
  Quelle: OWASP ASVS V2 (Authentication) § „Use a centralized identity provider";
  BSI APP.4.4.A3 (Identitätsmanagement).

- **[HIGH]** **`:latest`-Tags in 4 Compose-Files.** Reproduzierbares Deployment ist
  damit nicht gegeben; im Phase-2-Gate G2.4 ist „gepinnte Image-Tags" Pflicht.
  Wirkt sich besonders beim VPN-Sidecar aus, der `cap_add: NET_ADMIN`/`SYS_ADMIN`
  hält (also größere Privilegien). Bezug: **Sicherheit + Idempotenz**.
  Quelle: NIST SP 800-190 §4.5.2 („Use minimal base images and tag images
  immutably"), CIS Docker Benchmark §4.2 („Use trusted base images"), §5.3.

- **[HIGH]** **`inventory/secrets/.sops.yaml` enthält keine echten Recipients (zwei
  `# TODO`-Stellen).** Folge: aus dem Repo kann niemand reproduzierbar deployen,
  weil die `.enc`-Files erst gegen einen VPS-Host-Pubkey verschlüsselt werden
  müssen, der nicht im Repo dokumentiert ist. Verstößt gegen NFR-7 („GitOps:
  außer dem age-Privatkey ist der gesamte System-Zustand im Repo nachvollziehbar")
  des Projekts selbst. Bezug: **Dokumentation + Sicherheit (Auditierbarkeit)**.
  Quelle: BSI ORP.4.A22 (Geregeltes Schlüsselmanagement), `shared/target-state.md`
  § „Secrets nie im Klartext".

- **[MEDIUM]** **Container-Hardening nur in der VPS-Variante, nicht im Standalone-
  Compose.** Wenn der Standalone-Modus produktiv genutzt wird, fehlen
  `read_only`, `cap_drop:ALL`, `no-new-privileges`, mem/pids-Limits. Bezug:
  **Sicherheit**. Quelle: NIST SP 800-190 §4.5 (Container Runtime Security),
  CIS Docker §5.3 („Restrict Linux Kernel Capabilities"), §5.25, §5.28.

- **[MEDIUM]** **Kein Repo-internes ADR für vielzählige Architekturentscheidungen.**
  Sechs Entscheidungen sind im Projekt getroffen worden (Stack-Wahl, SQLite+YAML-
  Dualität, VPN-Provider, authgate, Build-Host, Image-Build on host) ohne ADR-
  Spur. Bezug: **Dokumentation + Komplexität**. Quelle: `shared/target-state.md`
  § „ADR-Pflicht für nicht-triviale Architekturentscheidungen"; ISO/IEC/IEEE 42010
  („documented architecture decisions").

- **[MEDIUM]** **EU Data Act nicht referenziert.** Das System verarbeitet IoT-
  Daten vernetzter Geräte – Datenverordnung 2023/2854 Kap. II ist seit
  12.09.2025 anwendbar. Bisher fehlt im Projekt jede Erwähnung; ein
  Repo-internes ADR mit Artikel-Mapping ist überfällig (verlangt von
  `shared/standards/eu-data-act.md` § „ADR-Pflicht"). Bezug: **Sicherheit (im
  Sinne Compliance) + Dokumentation**. Quelle: VO (EU) 2023/2854, Art. 3–7,
  Art. 13, Art. 23–31; `shared/standards/eu-data-act.md`.

- **[MEDIUM]** **Beobachtbarkeit unter Cockpit-Ziel.** `shared/target-state.md`
  § „Beobachtbarkeit als Pflicht" und Quality-Gate G3.5 verlangen Prometheus-
  Metriken pro Projekt. Im Repo: `/health`-Endpoint vorhanden, kein
  `/metrics`-Endpoint, keine strukturierten Logs (nur `eprintln!`/`println!`).
  Bezug: **Dokumentation + Operability**. Quelle: Google SRE Book Kap. 6
  („Monitoring Distributed Systems"), Prometheus Best Practices.

- **[MEDIUM]** **EU-KI-VO-Status nicht dokumentiert.** Auch ein „Nicht betroffen"
  braucht einen Repo-internen Eintrag mit Artikelangabe (Art. 3 Abs. 1,
  Anhang III), so verlangt es `shared/standards/eu-ai-act.md` § Audit-Hinweis.
  Heute fehlt der. Bezug: **Dokumentation + Compliance-Vorlauf**. Quelle: VO (EU)
  2024/1689, Art. 3 Abs. 1.

- **[LOW]** **Hostname-Reuse `vps.example.org` für unzusammenhängenden
  Dienst.** Der VPS-VPS heißt im DNS nach einem anderen, nicht laufenden
  Dienst. Schafft kognitive Reibung in Notfällen und Runbooks. Bezug:
  **Dokumentation**. Quelle: gelebte Praxis, keine harte Norm.

- **[LOW]** **Lizenz-Inkonsistenz.** `Cargo.toml` deklariert `license = "UNLICENSED"`,
  `README.md` schreibt „privat / TBD", Dockerfile-Label sagt
  `org.opencontainers.image.licenses="MIT"`, `LICENSE`-Datei im Repo existiert.
  Bezug: **Dokumentation**. Quelle: SPDX / Repository-Hygiene.

- **[LOW]** **Kein Backup-Plan für SQLite + Compose-State.** `docs/vps-setup.md`
  listet ihn als „post-V1". Solange nichts produktiv läuft, ist das vertretbar,
  aber spätestens Phase-2-Gate G2.6 verlangt eine getestete Backup-Routine. Bezug:
  **Operability**. Quelle: BSI CON.3 (Datensicherungskonzept), ISO/IEC 27001 A.12.3.

## Knowledge

### Thema: Adapter-Pattern für Infrastruktur-Komponenten (VPN-Sidecar)

**Prinzip**: Wenn eine Anwendung mit mehreren austauschbaren Infrastruktur-
Anbietern (VPN, Speicher, Identity, Message Bus) sprechen muss, gehört der
Provider-Code **nicht** in die Anwendung. Stattdessen definiert die Anwendung
einen kanonischen Vertrag (hier: ein Sidecar mit dem fixen Namen `vpn`, dessen
Netzwerk-Namespace die App joint), und jeder Provider liefert eine
austauschbare Adapter-Implementierung dieses Vertrags (hier: drei Compose-
Overlays). Das Adapter-Pattern (Gang-of-Four, „Structural Patterns") gilt
hier nicht für Code-Klassen, sondern für Deployment-Artefakte.

**Quelle**: Gamma et al., „Design Patterns" (1994), Adapter Pattern;
übertragen auf Container-Architektur in NIST SP 800-190 §3.1 („Microservices
isolation"). Kanonisches Beispiel: 12-Factor App, Faktor IV (Backing
Services).

**Anwendung hier**: Die App weiß nichts von Tailscale/NetBird/WireGuard.
Compose-Overlays liefern den Adapter. Das ist sauber – ein Pattern, das ich als
**Pattern of Praise** für andere Cockpit-Projekte hervorhebe (siehe Praise).

**Vertiefung**: Stichwort „Sidecar Pattern" (Bilgin Ibryam, „Kubernetes
Patterns", Kapitel „Init Containers" und „Sidecar").

### Thema: GitOps und die Pflicht zur Reproduzierbarkeit

**Prinzip**: Ein GitOps-fähiges Repository muss aus sich heraus deploybar sein
– bis auf genau einen klar benannten Out-of-Band-Geheimwert (üblich: ein
Recovery-Key). Wenn das Repository unmarkierte `# TODO`-Platzhalter für
Recipient-Keys enthält, ist die Reproduzierbarkeit gebrochen, weil ein
Neuaufsetzer raten muss, welcher Pubkey rein soll. Das verstößt gegen das
Prinzip „Doku ist Code" und untergräbt Auditierbarkeit. Die Lösung: entweder
echte Pubkey-Werte einchecken (Pubkeys sind nicht geheim) oder den fehlenden
Wert explizit als „muss vor erstem Sync erzeugt werden, Quelle: `<wo>`"
dokumentieren.

**Quelle**: Weaveworks „GitOps Principles" (deklarativ, versioniert,
automatisch eingespielt, kontinuierlich abgeglichen);
BSI ORP.4.A22 („Geregeltes Schlüsselmanagement"); `shared/target-state.md`
§ „Idempotenz" und § „Doku ist Code".

**Anwendung hier**: `inventory/secrets/.sops.yaml` zeigt zwei TODOs für den
VPS-Host-Pubkey. Das ist genau der typische GitOps-Reproduzierbarkeits-
Bruch.

**Vertiefung**: „GitOps and Kubernetes" (Yuen et al., Manning 2021), Kap. 3
„Trust and Secrets".

### Thema: KIC-Konvergenz – Negativabgrenzung als ADR-Pflicht

**Prinzip**: Auch eine App, die heute **keine** KI-Komponenten enthält, muss
ihren KI-VO-Status (Anwendungsbereich Art. 3 Abs. 1, ggf. Anhang III) im
Repo dokumentieren. „Nicht anwendbar" ist eine Architekturentscheidung wie
jede andere – sie kann sich morgen ändern, wenn der Roadmap-Schritt Phase 5
(Automationen) Modelle einbindet. Ein expliziter Negativ-ADR mit Artikelbezug
ist günstiger zu pflegen als ein nachträgliches Konformitäts-Audit.

**Quelle**: VO (EU) 2024/1689 (KI-VO), Art. 3 Abs. 1 (Definition KI-System),
Art. 6 + Anhang III (Klassifikation Hochrisiko); `shared/standards/eu-ai-act.md`
§ Audit-Hinweis („Wir sind nicht betroffen" ist nur mit konkretem Artikelbezug
valide).

**Anwendung hier**: Heute fehlt der Negativ-ADR. Phase 5 (Automation-Migration)
könnte das in Zukunft kippen; eine deterministische Regelautomation (HA-Trigger
→ Aktion) fällt sehr wahrscheinlich nicht unter Art. 3 Abs. 1, aber sobald
ein lernendes System (z. B. „Anwesenheits-Prädiktion auf Basis von Mustern")
hinzukommt, schon.

**Vertiefung**: AI Act Explorer
(<https://artificialintelligenceact.eu/the-act/>), Suchbegriff „Definition of
AI system"; KIC-EIN-2.

## Recommendations

- **R1** (Adressat: User): **Cockpit-Eintrag `ha-automation` in `config/cockpit.yaml`
  korrigieren.** `tech_stack` ist nicht `[home-assistant, mqtt, zigbee]`, sondern
  z. B. `[rust, sqlite, docker, caddy, sops-age]` mit Anmerkung „observiert
  HA/CCU/Hue/Shelly via VPN". `type` bleibt `edge` (das passt, weil der Dienst
  Heim-Netz-Inspektion macht). Severity **HIGH**. Grund: ohne Korrektur planen
  Security- und SRE-Audit ihre Werkzeuge gegen die falsche Plattform.

- **R2** (Adressat: User + `architect`): **ADR-Anstoß SSO-Wahl.** ADR-Entwurf
  „SSO-Provider für ha-automation – Keycloak (Cockpit-Soll) vs. Authentik
  (Repo-Plan)". Pro/Contra mit Complexity, Security Impact, Migrationsaufwand.
  Empfehlung des Architekten: Cockpit-Soll Keycloak gewinnt, weil
  `target-state.md` § Architekturprinzip #1 das so vorsieht und Doppel-IDP
  Komplexität schafft; `authgate` bleibt Übergang. Severity **HIGH**. Wird in
  Phase 3 (SHARED) ohnehin akut, also lieber jetzt klären.

- **R3** (Adressat: `ha-automation-dev`): **`.sops.yaml`-TODOs schließen.** Echten
  VPS-Host-age-Pubkey einsetzen (ist nicht geheim) und mindestens eine
  Beispiel-`.env.enc` einchecken, damit Reproduzierbarkeit gegeben ist. Severity
  **HIGH**. Bezug: NFR-7 des Projekts und target-state-§ Doku-ist-Code.

- **R4** (Adressat: `ha-automation-dev`): **Pinned Image-Tags für alle Compose-
  Overlays.** Tailscale, NetBird, WireGuard, Caddy, sops jeweils Digest-pinnen
  (analog `docker-compose.vps.yml`). Bei Bedarf neue Versions-Datei
  `docker/versions.yml` als Single-Source. Severity **HIGH** (G2.4-Gate
  Phase 2). Quelle: NIST SP 800-190 §4.5.2.

- **R5** (Adressat: `ha-automation-dev`): **Container-Hardening auf Standalone-
  Compose erweitern.** `read_only: true`, `cap_drop: [ALL]`, `no-new-privileges`,
  tmpfs für `/tmp` und `/run/inventory`, mem/cpu/pids-Limits – analog zur
  VPS-Variante. VPN-Sidecars brauchen weiter `NET_ADMIN`; das ist ok, aber
  `cap_drop: ALL` + `cap_add: NET_ADMIN` ist konkreter als kein cap_drop.
  Severity **MEDIUM**. Quelle: CIS Docker Benchmark §5.3, §5.25, §5.28.

- **R6** (Adressat: `ha-automation-dev`): **Repo-internes ADR-Verzeichnis
  anlegen** und retroaktiv ADRs für die sechs erkannten Architekturentscheidungen
  schreiben (Stack-Wahl, SQLite+YAML-Dualität, VPN-Vertrag, authgate, Build-Host,
  Image-Build on host). Severity **MEDIUM**. Quelle: ISO/IEC/IEEE 42010 und
  target-state-§ ADR-Pflicht.

- **R7** (Adressat: `ha-automation-dev` + `compliance`): **Negativ-ADR EU KI-VO +
  Mapping-ADR EU Data Act** schreiben. KI-VO: „Nicht anwendbar gemäß
  Art. 3 Abs. 1 / Anhang III, weil deterministische Inventarisierung ohne
  Lernverfahren; Re-Check bei Phase 5". Data Act: Mapping auf Art. 3–7 (IoT-
  Datenzugang, Endnutzerrechte am Heim-Netz-Inhaber-Inventar), Art. 23–31
  (Backup-Switching auf S3). Severity **MEDIUM**. Quelle:
  `shared/standards/eu-ai-act.md`, `shared/standards/eu-data-act.md`.

- **R8** (Adressat: `sre` + `ha-automation-dev`): **Metrics-Endpoint + strukturierte
  Logs**. `/metrics` für Prometheus (Counter pro Sync-Quelle, Histogram für
  Sync-Dauer, Gauge für `firmware_snapshot`-Inserts pro Lauf), Logs in
  JSON-Lines. Severity **MEDIUM** (greift erst in Phase 3 G3.5, aber Vorlauf
  spart Refactor).

- **R9** (Adressat: `ha-automation-dev` + `docs`): **Lizenz-Inkonsistenz
  auflösen** (`Cargo.toml` „UNLICENSED" vs. Image-Label „MIT" vs. README
  „privat/TBD"). Severity **LOW**, aber leicht zu fixen.

- **R10** (Adressat: `architect` – ich selbst, Folge-Iteration): **ADR-Entwurf
  Phase-5-Trigger.** Sobald die ersten Automationen aus HA/Node-RED migriert
  werden, einen Check gegen EU-KI-VO-Art-3-Definition aktivieren. Severity
  **LOW** (jetzt), aber als Vormerkung im Iterations-Plan.

## Praise

- **Sehr saubere Trennung zwischen App und Infrastruktur.** Die VPN-Abstraktion
  über `network_mode: service:vpn` und drei austauschbare Overlays ist
  vorbildlich. Ich empfehle, das Pattern auch in `vps-stack` zu spiegeln, wo
  es ähnliche Substitutionen (Reverse Proxy, IDP, Backup-Target) geben wird.

- **Auth-Sidecar mit Vertrags-Stabilität.** Der Header-Vertrag
  `X-Authentik-Username` und die Tatsache, dass `authgate` und das künftige
  Authentik denselben Vertrag bedienen, ist eine saubere Anwendung des
  Liskov-Substitutions-Prinzips auf Deployment-Ebene. Wechsel ist Einzeiler im
  Caddyfile.

- **Krypto im `authgate` ist nach Lehrbuch.** PBKDF2 mit OWASP-2023-Rundenzahl,
  konstantzeitiger Vergleich, Dummy-Hash gegen Enumeration, signiertes
  Stateless-Cookie mit Versions-Präfix, Open-Redirect-Schutz. Das ist Material
  für einen späteren Architektur-Knowledge-Eintrag.

- **Dokumentations-Schichtung.** README → requirements → architecture → roadmap
  → vps-setup ineinander verzahnt, jeweils mit ASCII-Diagrammen und
  Trust-Boundaries. Vorbildlich für Cockpit-weite Konvention.

- **Idempotenz konsequent.** Migrations idempotent, Upsert mit `ON CONFLICT`,
  YAML-Export deterministisch sortiert, `git_publish` no-op bei sauberer Working
  Copy. Das macht spätere SRE-Arbeit (Cronjob-Wiederholung, Backup-Replay)
  einfach.

## Gate-Bezug

- **G1.1** (mindestens ein Audit-Report pro aktivem Experten): teilweise erfüllt
  durch diesen Report – noch ausstehend: `security`, `sre`, `docs`.
  Status: **teilweise – läuft**.
- **G1.2** (CRITICAL dokumentiert und priorisiert): **n/a – keine CRITICAL-
  Findings aus dieser Architekt-Sicht**. STOPP-Bedingung H.1 nicht ausgelöst.
- **G1.3** (README aktuell, ≤ 14 Tage): erfüllt – `projects/ha-automation/README.md`
  vom 2026-05-16, Stand „Phase 1+2+3 abgeschlossen". Status: **erfüllt**.
- **G1.4** (Architektur-Skizze): erfüllt – `docs/architecture.md` mit ASCII-
  Diagramm, Lesepfad, Userpfad, Trust Boundaries, Secrets-Architektur, Build-
  und Deploy-Pfad. Status: **erfüllt**.
- **G1.5** (Bias-Reflexion in jedem Audit): erfüllt durch eigene Sektion unten.
  Status: **erfüllt**.

Cross-Phase-Gates:

- **H.1** (STOPP-Taste): nicht ausgelöst – kein CRITICAL.
- **H.2** (irreversible Aktionen): nicht zutreffend – Audit ist read-only.

## Bias-Reflexion (Pflicht G1.5)

Im Sinne von KIC-EIN-2 und EU KI-VO Art. 14 reflektiere ich, wo ich beim
Audit Gefahr lief, einer Vor-Annahme oder einer früheren KI-/Sub-Agent-
Aussage zu schnell zu folgen.

**Wo Automatisierungs- und Confirmation-Bias drohten**:

1. **Stack-Fehlbeschreibung in `cockpit.yaml`.** Ich hatte beim Lesen von
   `cockpit.yaml` den Eintrag „edge, [home-assistant, mqtt, zigbee]" gesehen
   und war kurz davor, in der nächsten Datei nach `configuration.yaml` zu
   suchen statt zuerst das Repo zu listen. Das wäre Confirmation-Bias
   gewesen – ich hätte den Cockpit-Eintrag bestätigt, statt ihn zu prüfen.
   **Gegenmaßnahme**: ich habe vor dem Inhalt-Lesen `tree`/`find` auf das
   Repo gemacht und dabei festgestellt, dass es ein Rust-Projekt ist. Das
   Finding 1 ist genau das Ergebnis dieser Korrektur.

2. **„Phase 1+2+3 abgeschlossen, 52 Tests grün"-Bias.** Das Projekt deklariert
   in README und Memory Phase 1+2+3 als fertig. Das ist eine starke
   Behauptung, die mich zu der Annahme verführen könnte, „dann ist alles
   gut". Tatsächlich beziehen sich die 52 Tests auf die Bibliothek mit Mocks
   – Live-Smoke-Tests gegen echte Heim-Systeme sind vertagt, und im
   Deployment-Pfad (Phase 4) sind noch zentrale Härtungs- und Auth-Items
   offen. **Gegenmaßnahme**: ich habe `docs/roadmap.md` Zeile für Zeile mit
   dem realen Compose-Files-Stand abgeglichen und die Diskrepanz „authgate
   da, Tests grün, aber VPS-Build/Test noch offen" erfasst.

3. **Vertrauen in den Auth-Sidecar-Code.** `authgate` sieht handwerklich sehr
   sauber aus (PBKDF2-Rundenzahl, konstantzeitiger Vergleich, Open-Redirect-
   Schutz). Risiko: ich übersehe einen architektonischen Fehler, weil der
   Krypto-Code mich beeindruckt. **Gegenmaßnahme**: ich habe explizit den
   architektonischen Blickwinkel angelegt (Doppel-IDP zum Cockpit-Soll) und
   nicht die Krypto-Detailprüfung – die gehört zum `security`-Experten und
   ist nicht meine Zuständigkeit.

4. **Optimismus durch Dokumentationsqualität.** Die Projekt-Doku ist
   ungewöhnlich gut. Das hätte mich verleiten können, weniger streng zu
   suchen. **Gegenmaßnahme**: ich habe die Befunde 4 (SPoF), 6 (.sops.yaml-
   TODO), 7 (`:latest`-Tags), 9 (Hardening-Asymmetrie), 10 (kein CI)
   ausdrücklich aus dem Maschinen-/Compose-Material gezogen, nicht aus der
   selbstbeschreibenden Doku.

**Was gegengeprüft werden sollte**:

- Mein Urteil „KI-VO nicht anwendbar" stützt sich auf den heutigen Code-
  Stand. **Bitte gegenchecken durch `compliance`**, ob die geplante
  Automation-Migration in Phase 5 oder die HA-Anbindung selbst (HA enthält
  inzwischen optionale ML-Add-ons) den Status kippt. Quelle für die
  Abgrenzung: EU KI-VO Art. 3 Abs. 1.

- Mein Urteil „Authentik vs. Keycloak – Keycloak gewinnt durch Cockpit-Soll"
  beruht auf der Lesart der `target-state.md`. **Bitte gegenchecken durch
  `security` und den User**, ob ggf. eine bestehende, betriebsfertige
  Authentik-Instanz (Repo erwähnt „bestehende Instanz") das umkehrt – das
  wäre ein User-Eskalations-Fall.

- Mein Urteil „Standalone-Compose ohne Hardening ist MEDIUM" ist
  kontextabhängig: wenn der Standalone-Modus tatsächlich nur in
  Test-/Smoke-Umgebungen eingesetzt wird, ist die Schwere geringer. **Bitte
  gegenchecken** durch User-Aussage, ob Standalone produktiv landet oder
  reines Entwickler-Konstrukt bleibt.
