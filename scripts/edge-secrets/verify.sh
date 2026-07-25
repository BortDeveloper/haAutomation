#!/bin/sh
# ha-automation Edge-Secret-Verifikation (Cockpit-ADR-0004 §1/§3/§5).
#
# Prüft die Chiffrat-Disziplin unter edge-secrets/ — OHNE private Schlüssel,
# damit CI-/Hook-tauglich (S19-Vorbereitung).
#
# Aufruf:
#   scripts/edge-secrets/verify.sh [datei.enc ...]
#
# Ohne Argumente werden alle Dateien unter edge-secrets/ geprüft
# (ausgenommen .sops.yaml, README.md und recipients/).
#
# Prüfungen je Datei:
#   V1  Datei endet auf .enc (alles andere unter edge-secrets/ = Verstoß)
#   V2  sops-Chiffrat-Signatur vorhanden (ENC[AES256_GCM + sops-Metadaten)
#   V3  GENAU 3 age-Recipients in den sops-Metadaten (Eskrow, ADR-0004 §3)
#   V4  bekannte Edge-Marker-Schlüssel tragen keinen Klartext-Wert
# Zusätzlich (Gesamtstatus):
#   V5  Recipient-Stand: sobald Chiffrate existieren, müssen 3 Pubkeys unter
#       recipients/ liegen und die TODOs in .sops.yaml getilgt sein.
#
# Exit-Code 0 = alles sauber, 1 = mindestens ein Verstoß.
set -eu

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
edge_dir="$repo_root/edge-secrets"
sops_config="$edge_dir/.sops.yaml"

fail=0
err()  { echo "VERSTOSS: $*" >&2; fail=1; }
info() { echo "verify: $*"; }

# --- Dateiliste bestimmen ----------------------------------------------------
if [ $# -gt 0 ]; then
  files=$*
else
  files=$(find "$edge_dir" -type f \
            ! -path '*/recipients/*' \
            ! -name '.sops.yaml' \
            ! -name 'README.md' 2>/dev/null | sort)
fi

enc_count=0
for f in $files; do
  [ -f "$f" ] || { err "Datei nicht gefunden: $f"; continue; }

  # V1: nur .enc unter edge-secrets/
  case "$f" in
    *.enc) ;;
    *) err "V1 $f: keine .enc-Datei — unter edge-secrets/ ist nur Chiffrat zulässig."
       continue ;;
  esac
  enc_count=$((enc_count + 1))

  # V2: sops-Signatur
  if ! grep -q 'ENC\[AES256_GCM' "$f"; then
    err "V2 $f: keine ENC[AES256_GCM-Signatur — kein sops-Chiffrat?"
  fi
  if ! grep -Eq '(^sops:|"sops":|^sops_|\[sops\])' "$f"; then
    err "V2 $f: keine sops-Metadaten gefunden."
  fi

  # V3: genau 3 age-Recipients (YAML: 'recipient: age1…' / JSON: '"recipient": "age1…')
  n=$(grep -Ec 'recipient"?:[[:space:]]*"?age1' "$f") || n=0
  if [ "$n" -ne 3 ]; then
    err "V3 $f: erwartet GENAU 3 age-Recipients (ADR-0004 §3), gefunden: $n"
  fi

  # V4: Marker-Schlüssel dürfen nur ENC[-Werte tragen (sops lässt Keys im Klartext)
  if grep -E '(network_key|trust_center_link_key)"?[[:space:]]*:' "$f" \
     | grep -Ev 'ENC\[' >/dev/null 2>&1; then
    err "V4 $f: Edge-Marker-Schlüssel mit Nicht-ENC[-Wert — Klartext-Verdacht."
  fi
done

# --- V5: Recipient-Gesamtstand ------------------------------------------------
pub_count=0
for p in "$edge_dir"/recipients/*.pub; do
  [ -f "$p" ] || continue
  if grep -Eq '^age1[0-9a-z]+$' "$p"; then
    pub_count=$((pub_count + 1))
  else
    err "V5 $p: enthält keine einzelne age1…-Zeile (Pubkey-Format)."
  fi
done

todo_open=0
if grep -q 'TODO(recipients)' "$sops_config" 2>/dev/null; then
  todo_open=1
fi

if [ "$enc_count" -gt 0 ]; then
  [ "$pub_count" -eq 3 ] || err "V5: $enc_count Chiffrat(e) vorhanden, aber $pub_count/3 Pubkeys unter recipients/."
  [ "$todo_open" -eq 0 ] || err "V5: Chiffrate vorhanden, aber .sops.yaml trägt noch TODO(recipients)."
else
  if [ "$todo_open" -eq 1 ] || [ "$pub_count" -lt 3 ]; then
    info "HINWEIS: noch keine Chiffrate; Recipients unvollständig ($pub_count/3, \
TODO offen: $todo_open) — Operator-Checkliste docs/runbooks/operator-checklist-adr-0004.md \
(R-SOPS-RECIPIENTS-TODO)."
  fi
fi

if [ "$fail" -eq 0 ]; then
  info "OK — geprüfte Chiffrate: $enc_count, Pubkeys: $pub_count/3."
else
  echo "verify: FEHLGESCHLAGEN (Details oben)." >&2
fi
exit "$fail"
