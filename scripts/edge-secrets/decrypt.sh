#!/bin/sh
# ha-automation Edge-Secret-Entschlüsselung — Restore-Pfad (Cockpit-ADR-0004 §6).
#
# Aufruf:
#   scripts/edge-secrets/decrypt.sh <datei.enc> [ziel-datei]
#
# Ohne Ziel-Datei geht der Klartext auf stdout (bevorzugt: nichts landet auf
# der Platte, direkt weiterpipen). Ein Ziel INNERHALB des Repos wird
# verweigert — Klartext gehört nie in den Working Tree.
#
# Key-Quellen (die erste lesbare gewinnt, sofern SOPS_AGE_KEY_FILE nicht
# bereits gesetzt ist):
#   1. $SOPS_AGE_KEY_FILE            (explizit — z. B. für den Drill)
#   2. /etc/inventory/age.key        (Edge-Host-Recipient, HAOS)
#   3. /etc/backup/age.key           (Backup-Operator-Recipient, VPS-Host)
#
# DR-Hardware-Token (3. Recipient): age-plugin-yubikey muss im PATH liegen;
# SOPS_AGE_KEY_FILE auf die Identity-Datei des Tokens setzen
# (`age-plugin-yubikey --identity`). Details im Runbook
# docs/runbooks/edge-secret-backup.md §Restore.
set -eu

die() { echo "decrypt.sh FEHLER: $*" >&2; exit 1; }

[ $# -ge 1 ] && [ $# -le 2 ] || die "Aufruf: decrypt.sh <datei.enc> [ziel-datei]"

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)

src=$1
[ -f "$src" ] || die "Chiffrat nicht gefunden: $src"
command -v sops >/dev/null 2>&1 || die "'sops' ist nicht installiert/im PATH."

if [ -z "${SOPS_AGE_KEY_FILE:-}" ]; then
  for k in /etc/inventory/age.key /etc/backup/age.key; do
    if [ -r "$k" ]; then
      SOPS_AGE_KEY_FILE=$k
      export SOPS_AGE_KEY_FILE
      break
    fi
  done
fi
if [ -z "${SOPS_AGE_KEY_FILE:-}" ]; then
  echo "decrypt.sh HINWEIS: kein age-Privatkey gefunden (SOPS_AGE_KEY_FILE ungesetzt," >&2
  echo "  /etc/inventory/age.key und /etc/backup/age.key nicht lesbar)." >&2
  echo "  sops wird es dennoch versuchen (z. B. via age-plugin-yubikey / keys.txt)." >&2
fi

# Datentyp aus dem Namen (ohne .enc) ableiten — .enc kennt sops nicht,
# ohne Angabe würde es den Store-Typ falsch raten.
base=${src%.enc}
case "$base" in
  *.json)        ftype=json ;;
  *.yaml|*.yml)  ftype=yaml ;;
  *.env)         ftype=dotenv ;;
  *)             ftype=binary ;;
esac

if [ $# -eq 2 ]; then
  dest=$2
  dest_dir=$(dirname "$dest")
  mkdir -p "$dest_dir"
  abs_dest_dir=$(cd "$dest_dir" && pwd)
  case "$abs_dest_dir" in
    "$repo_root"|"$repo_root"/*)
      die "Ziel liegt im Repo ($abs_dest_dir) — Klartext gehört nie in den \
Working Tree. Außerhalb entschlüsseln (z. B. /tmp) und nach Gebrauch shred -u."
      ;;
  esac
  sops --decrypt --input-type "$ftype" --output-type "$ftype" \
       --output "$dest" "$src"
  chmod 600 "$dest" 2>/dev/null || true
  echo "OK: Klartext unter $dest (Mode 600). Nach Gebrauch: shred -u '$dest'." >&2
else
  exec sops --decrypt --input-type "$ftype" --output-type "$ftype" "$src"
fi
