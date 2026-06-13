#!/usr/bin/env bash
# sync-ha.sh — Fokussierter Runner: liest das Geraete-Inventar einer
# Home-Assistant-Instanz (GET /api/states) und schreibt es ins Inventory
# (SQLite-Upsert + YAML-Snapshot).
#
# Im Gegensatz zu smoke-test.sh laeuft dieses Script NUR den HA-Sync und
# normalisiert HA_URL defensiv — es fixt den frueher beobachteten
# Doppel-Praefix-Bug ("http://http://...") aus handgeschriebenen Runnern.
#
# Die eigentliche Logik liegt in der Binary (`inventory sync ha`,
# src/sync/ha.rs). Dieses Script ruft sie nur korrekt auf — es schreibt
# selbst NICHT in DB/YAML, damit die Owner-Logik der Binary die einzige
# Schreib-Stelle bleibt.
#
# Verwendung:
#   1. cargo build --release --locked --bin inventory
#   2. cp test-setup.env.example local/test-setup.env  (falls noch nicht da)
#   3. local/test-setup.env: HA_URL + HA_TOKEN setzen, chmod 600
#   4. ./sync-ha.sh
#
# HA_URL darf mit oder ohne Schema angegeben werden:
#   homeassistant.local:8123        -> http://homeassistant.local:8123
#   http://10.0.0.5:8123            -> unveraendert
#   https://ha.example.local/       -> https://ha.example.local
#   https://ha.example.local/api    -> https://ha.example.local  (+ Warnung)
#
# Exit-Codes:
#   0   HA-Sync erfolgreich
#   1   Config-Fehler / fehlende Pflichtwerte
#   2   Binary nicht gefunden
#   3   HA-Sync fehlgeschlagen

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

ENV_FILE="local/test-setup.env"
BIN="./target/release/inventory"
# Windows .exe-Suffix tolerieren (Git Bash / MSYS)
if [ ! -x "$BIN" ] && [ -x "$BIN.exe" ]; then
    BIN="$BIN.exe"
fi

if [ ! -x "$BIN" ]; then
    echo "ERROR: $BIN nicht gefunden / nicht ausfuehrbar."
    echo "  -> cargo build --release --locked --bin inventory"
    exit 2
fi

if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: $ENV_FILE fehlt."
    echo "  -> cp test-setup.env.example $ENV_FILE && chmod 600 $ENV_FILE"
    echo "  -> dann HA_URL + HA_TOKEN ausfuellen."
    exit 1
fi

# Sicherheits-Check: env-Datei sollte nicht world-readable sein
# (Git Bash / MSYS gibt hier ggf. 0644 zurueck — nur Warn, kein Stop).
PERMS=$(stat -c '%a' "$ENV_FILE" 2>/dev/null || stat -f '%A' "$ENV_FILE" 2>/dev/null || echo "?")
case "$PERMS" in
    600|400|?) ;;
    *) echo "WARN: $ENV_FILE hat Permissions $PERMS — empfohlen 600." ;;
esac

# Env laden (set -a exportiert alle folgenden Variablen automatisch)
set -a
# shellcheck disable=SC1090
. "$ENV_FILE"
set +a

# Pflichtwerte pruefen
if [ -z "${HA_URL:-}" ] || [ -z "${HA_TOKEN:-}" ]; then
    echo "ERROR: HA_URL und HA_TOKEN muessen in $ENV_FILE gesetzt sein."
    exit 1
fi

# --- HA_URL defensiv normalisieren ----------------------------------
# 1) Whitespace trimmen
HA_URL="$(printf '%s' "$HA_URL" | tr -d '[:space:]')"
# 2) Schema sicherstellen — KEIN Doppel-Praefix wenn schon vorhanden
case "$HA_URL" in
    http://*|https://*) ;;
    *)
        echo "INFO: HA_URL ohne Schema — ergaenze http:// -> http://$HA_URL"
        HA_URL="http://$HA_URL"
        ;;
esac
# 3) Trailing Slashes entfernen
HA_URL="${HA_URL%/}"
# 4) versehentliches /api-Suffix abschneiden (Binary haengt /api/states selbst an)
case "$HA_URL" in
    */api)
        echo "WARN: HA_URL endete auf /api — entferne Suffix (Binary haengt /api/states selbst an)."
        HA_URL="${HA_URL%/api}"
        ;;
esac
export HA_URL

# Publish-Guard: explizit AUS (Audit R-HIGH-3) — dieser Runner testet nur lokal.
unset INVENTORY_PUBLISH || true
unset INVENTORY_PUBLISH_CONFIRM || true

# Sandbox-Pfade anlegen
mkdir -p "$(dirname "${INVENTORY_DB:-./local/inventory.db}")"
mkdir -p "${INVENTORY_YAML_DIR:-./local/yaml}"

echo "=== 1/2: DB-Migrationen ==="
"$BIN" migrate
echo

echo "=== 2/2: Home Assistant Sync ($HA_URL) ==="
if "$BIN" sync ha; then
    echo "  -> HA sync OK"
else
    RC=$?
    echo "  -> HA sync FAILED (exit $RC)"
    exit 3
fi
echo

echo "=== Ergebnis-Artefakte ==="
echo "  DB:   ${INVENTORY_DB:-./local/inventory.db}"
echo "  YAML: ${INVENTORY_YAML_DIR:-./local/yaml}/"
ls -la "${INVENTORY_YAML_DIR:-./local/yaml}/" 2>/dev/null || true
echo
echo "HA-Sync gruen."
