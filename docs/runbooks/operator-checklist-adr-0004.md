# Operator-Checkliste: age-Recipients erzeugen + eintragen (ADR-0004)

**Wer:** Operator/User (Hardware- und Host-Zugriff nötig — kein Agenten-Akt).
**Wann:** einmalig zur Inbetriebnahme; danach jährlich (Q1) bzw.
ereignisbasiert (Rotation, siehe [`edge-secret-backup.md`](edge-secret-backup.md) §7).

**Grundsatz:** Private Keys entstehen **auf dem jeweiligen Zielhost bzw.
Token** und verlassen ihn nie. Ins Repo kommen ausschließlich die
Public Keys (`age1…`) unter `edge-secrets/recipients/`.
**Keine Platzhalter-/Dummy-Keys** — solange Recipients fehlen, bleibt die
Pipeline bewusst fail-closed.

---

## Schritt 0 — Beschaffung / Standort (einmalig)

- [ ] DR-Hardware-Token beschaffen (YubiKey 5 o. ä., PIV-fähig).
- [ ] Off-Site-Verwahrort festlegen: anderer Brandabschnitt oder
      Bankschließfach (BSI CON.3.A4 — nicht im selben
      Brand-/Diebstahl-Ereignis wie der Edge).

## Schritt 1 — Edge-Host-Recipient (auf HAOS)

```sh
# auf dem Edge-Host (HAOS / Advanced SSH):
age-keygen -o /etc/inventory/age.key
chmod 0400 /etc/inventory/age.key
chown root:root /etc/inventory/age.key

# Public Key auslesen (dieser Wert kommt ins Repo):
age-keygen -y /etc/inventory/age.key
```

- [ ] Ausgabe (`age1…`) als **eine Zeile** in
      `edge-secrets/recipients/edge-host-<YYYY-MM-DD>.pub` speichern.

## Schritt 2 — Backup-Operator-Recipient (auf dem VPS-Stack-Host)

```sh
# auf dem VPS-Host:
age-keygen -o /etc/backup/age.key
chmod 0400 /etc/backup/age.key
chown root:root /etc/backup/age.key
age-keygen -y /etc/backup/age.key
```

- [ ] Ausgabe als `edge-secrets/recipients/backup-operator-<YYYY-MM-DD>.pub`.

## Schritt 3 — DR-Hardware-Token-Recipient (YubiKey)

> Hardware-Interaktion (`ykman`/`age-plugin-yubikey`) nur mit explizitem
> User-Konsens — Briefing-Vorgabe.

```sh
# auf einer Arbeitsstation mit eingestecktem Token:
age-plugin-yubikey --generate        # interaktiv: Slot, PIN- und Touch-Policy
# Recipient anzeigen (age1yubikey1…):
age-plugin-yubikey --list
# Identity-Datei für spätere Restores sichern (enthält KEINEN Private Key,
# nur die Token-Referenz — trotzdem nicht ins Repo):
age-plugin-yubikey --identity > ~/dr-token-identity.txt
```

- [ ] Recipient-Wert (`age1yubikey1…`) als
      `edge-secrets/recipients/dr-token-<YYYY-MM-DD>.pub`.
- [ ] Token am Off-Site-Standort deponieren; PIN getrennt verwahren.

## Schritt 4 — Recipients ins Repo eintragen

- [ ] Die drei `.pub`-Dateien committen (`edge-secrets/recipients/`).
- [ ] Alle drei `age1…`-Werte in **beide** sops-Konfigurationen unter
      **jeder** `creation_rules`-Regel eintragen und die
      `# TODO(recipients)`-Kommentare entfernen:
  - `edge-secrets/.sops.yaml` (Edge-Klasse — maßgeblich)
  - `home-inventory/secrets/.sops.yaml` (Betriebssecrets des eingefrorenen
    Inventory-Tools; mitpflegen, solange dessen `.enc`-Klassen existieren)

  Format je Regel — sops erwartet die Recipients als **einen
  Komma-String** (keine YAML-Liste):

  ```yaml
  - path_regex: coordinator_backup\.json\.enc$
    # edge-host-<datum>, backup-operator-<datum>, dr-token-<datum>
    age: "age1AAA...,age1BBB...,age1yubikey1CCC..."
  ```

- [ ] Falls bereits `.enc`-Dateien existieren (bei Rotation):
      `sops updatekeys <datei>.enc` auf jede Datei anwenden.
- [ ] `scripts/edge-secrets/verify.sh` ausführen → muss `OK` melden
      (Pubkeys 3/3, keine TODO-Reste bei vorhandenen Chiffraten).
- [ ] Commit + PR mit sichtbarem Diff.

## Schritt 5 — Erst-Backup ziehen

- [ ] Je Zeile der Scope-Tabelle in
      [`edge-secret-backup.md`](edge-secret-backup.md) §1: Klartext vom Edge
      holen → `scripts/edge-secrets/encrypt.sh` → committen → Klartext
      `shred -u`.
- [ ] Beim Backup-Lauf `sha256sum` der Klartexte notieren (Referenz für den
      Drill-Vollständigkeits-Hash; Notiz NICHT ins Repo, sie beschreibt
      Secret-Inhalte).

## Schritt 6 — Restore-Drill (schließt den G2.6-Beleg)

- [ ] Drill nach [`edge-secret-backup.md`](edge-secret-backup.md) §6 über
      **alle drei** Recipient-Pfade.
- [ ] Beleg-Datei in
      `stack-master/shared/audit-log/YYYY-MM-DD-ha-automation-restore-drill.md`
      (Datum, Operator, Recipient, RTO, Vollständigkeits-Hash).

## Schritt 7 — Meldung

- [ ] Vollzug an den Orchestrator melden → Abschluss-Audit
      (security + sre) → **H.1-Deaktivierung ist Orchestrator-Akt**,
      nicht Teil dieser Checkliste.

---

## Rotations-Kurzform (jährlich Q1 / ereignisbasiert)

1. Schritte 1–3 mit neuem Datum wiederholen (alte `.pub`-Dateien im Repo
   belassen bis zum Abschluss, dann entfernen — Git-Historie ist der Beleg).
2. Beide `.sops.yaml` auf die neuen Werte umstellen.
3. `sops updatekeys` auf **alle** `.enc`-Dateien.
4. `scripts/edge-secrets/verify.sh` + Drill über alle drei Pfade.
5. Alte Privatkeys auf den Hosts löschen (`shred -u`), Token-Slot neu belegen.
6. Rotations-Vermerk in `stack-master/shared/audit-log/`.
