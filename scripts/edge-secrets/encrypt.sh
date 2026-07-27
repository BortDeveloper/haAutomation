#!/bin/sh
# ha-automation Edge-Secret-Verschlüsselung (Cockpit-ADR-0004 §1/§3).
#
# Verschlüsselt genau EINE Klartext-Datei gegen die age-Recipients aus
# edge-secrets/.sops.yaml und legt das Chiffrat unter edge-secrets/ ab.
# INTERIM (ADR-0004 Nachtrag 2, 2026-07-27): 2 Recipients (Edge-Host +
# Backup-Operator); DR-Token wird bis 2026-08-15 nachgerüstet, danach
# Guards zurück auf ==3.
#
# Aufruf:
#   scripts/edge-secrets/encrypt.sh <klartext-datei> <ziel-pfad.enc>
#
# Beispiel:
#   scripts/edge-secrets/encrypt.sh /tmp/coordinator_backup.json \
#       edge-secrets/zigbee2mqtt/coordinator_backup.json.enc
#
# Eigenschaften:
#   - fail-closed: solange die Recipients in edge-secrets/.sops.yaml nicht
#     vollständig eingetragen sind (2x age1..., Interim s. o.), bricht das
#     Script ab — siehe docs/runbooks/operator-checklist-adr-0004.md.
#   - nutzt `sops --filename-override` (sops >= 3.9), damit die
#     .enc-orientierten path_regex-Regeln greifen; für älteres sops greift
#     ein Fallback über eine temporäre, zielgleich benannte Kopie
#     AUSSERHALB des Repos.
#   - verifiziert das Ergebnis sofort via verify.sh (sops-Header,
#     Recipient-Anzahl == 2, Interim).
#   - die Klartext-Quelle bleibt unangetastet; nach erfolgreichem Commit
#     sicher löschen (`shred -u <datei>` bzw. äquivalent).
set -eu

die() { echo "encrypt.sh FEHLER: $*" >&2; exit 1; }

[ $# -eq 2 ] || die "Aufruf: encrypt.sh <klartext-datei> <ziel-pfad.enc>"

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
sops_config="$repo_root/edge-secrets/.sops.yaml"

src=$1
dest=$2

[ -f "$src" ] || die "Quelle nicht gefunden: $src"
command -v sops >/dev/null 2>&1 || die "'sops' ist nicht installiert/im PATH."

case "$dest" in
  *.enc) ;;
  *) die "Ziel muss auf .enc enden (bekam: $dest)" ;;
esac

# Ziel auf einen absoluten Pfad unter <repo>/edge-secrets/ normalisieren.
case "$dest" in
  "$repo_root"/edge-secrets/*) abs_dest=$dest ;;
  edge-secrets/*)              abs_dest="$repo_root/$dest" ;;
  *) die "Ziel muss unter edge-secrets/ liegen (bekam: $dest)" ;;
esac
rel_dest=${abs_dest#"$repo_root"/}

# --- Fail-closed: Recipients müssen vollständig eingetragen sein ------------
if grep -q 'TODO(recipients)' "$sops_config"; then
  die "Recipients in edge-secrets/.sops.yaml sind noch TODO — \
Schlüssel-Erzeugung ist ein Operator-Akt: docs/runbooks/operator-checklist-adr-0004.md"
fi
# Recipients stehen je Regel als EIN Komma-String: age: "age1...,age1..."
# Regel-genauer Guard (ADR-0004 §3, Nachtrag 1 Auflage d): JEDE creation_rule
# muss GENAU 2 age-Recipients tragen — "== 2", nicht ">= 2"; eine Datei-
# Gesamtsumme würde leere oder überzählige Einzel-Regeln durchlassen.
# INTERIM per Nachtrag 2 (2026-07-27): ==2 statt ==3; bei DR-Token-
# Nachrüstung (bis 2026-08-15) zurück auf ==3 stellen ({1} -> {2} im Regex).
age_lines=$(grep -v '^[[:space:]]*#' "$sops_config" \
              | grep -E '^[[:space:]]*age:' || :)
[ -n "$age_lines" ] || die "keine age:-Regel in edge-secrets/.sops.yaml gefunden."
bad_rules=$(printf '%s\n' "$age_lines" \
              | grep -Evc '^[[:space:]]*age:[[:space:]]*"age1[0-9a-z]+(,[[:space:]]*age1[0-9a-z]+){1}"[[:space:]]*$' || :)
[ "$bad_rules" -eq 0 ] || die "$bad_rules creation_rule(s) in \
edge-secrets/.sops.yaml ohne GENAU 2 age-Recipients — Interim-Soll nach ADR-0004 Nachtrag 2 (2026-07-27)."

mkdir -p "$(dirname "$abs_dest")"

# Datentyp aus dem Zielnamen (ohne .enc) ableiten — steuert, wie sops die
# Datei parst (Keys bleiben lesbar, Werte werden ENC[...]; unbekannt = binary).
base=${rel_dest%.enc}
case "$base" in
  *.json)        ftype=json ;;
  *.yaml|*.yml)  ftype=yaml ;;
  *.env)         ftype=dotenv ;;
  *)             ftype=binary ;;
esac

if sops --help 2>&1 | grep -q 'filename-override'; then
  # sops >= 3.9: Regel-Matching direkt auf den Zielnamen
  sops --config "$sops_config" --encrypt \
       --filename-override "$rel_dest" \
       --input-type "$ftype" --output-type "$ftype" \
       --output "$abs_dest" "$src"
else
  # Fallback (z. B. sops 3.7.x): temporäre Kopie AUSSERHALB des Repos unter
  # dem Zielnamen, damit die path_regex-Regeln greifen.
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT INT TERM
  mkdir -p "$tmpdir/$(dirname "$rel_dest")"
  cp "$src" "$tmpdir/$rel_dest"
  sops --config "$sops_config" --encrypt \
       --input-type "$ftype" --output-type "$ftype" \
       --output "$abs_dest" "$tmpdir/$rel_dest"
  rm -rf "$tmpdir"
  trap - EXIT INT TERM
fi

# Sofort-Verifikation (sops-Header, Recipient-Anzahl, keine Klartext-Marker).
"$script_dir/verify.sh" "$abs_dest"

echo "OK: $rel_dest verschlüsselt und verifiziert."
echo "Nächste Schritte: git add '$rel_dest' && git commit; danach Klartext-Quelle"
echo "sicher löschen: shred -u '$src' (bzw. äquivalent)."
