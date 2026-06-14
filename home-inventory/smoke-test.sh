#!/usr/bin/env bash
# smoke-test.sh — Erster Verbindungstest gegen Test-HA + Test-CCU.
#
# Liest local/test-setup.env (siehe test-setup.env.example), checkt
# Pflichtwerte und fuehrt:
#   1. `home-inventory migrate`               (SQLite anlegen)
#   2. `home-inventory sync ha`               (sofern HA_URL+HA_TOKEN gesetzt)
#   3. `home-inventory sync ccu`              (sofern CCU_URL gesetzt)
#   4. `home-inventory sync nodered`          (sofern HA_URL+HA_TOKEN+NODERED_INGRESS_PATH gesetzt)
#
# aus. Die Binary muss vorher gebaut sein:
#   cargo build --release --locked --bin home-inventory
#
# Exit-Codes:
#   0   alle aktivierten Quellen erfolgreich
#   1   Config-Fehler / fehlende Pflichtwerte
#   2   Binary nicht gefunden
#   3+  Sync-Fehler (HA, CCU oder Node-RED)

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

ENV_FILE="local/test-setup.env"
BIN="./target/release/home-inventory"
# Windows .exe-Suffix tolerieren (Git Bash / MSYS)
if [ ! -x "$BIN" ] && [ -x "$BIN.exe" ]; then
    BIN="$BIN.exe"
fi

if [ ! -x "$BIN" ]; then
    echo "ERROR: $BIN nicht gefunden / nicht ausfuehrbar."
    echo "  -> cargo build --release --locked --bin home-inventory"
    exit 2
fi

if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: $ENV_FILE fehlt."
    echo "  -> cp test-setup.env.example $ENV_FILE && chmod 600 $ENV_FILE"
    echo "  -> dann die Werte (HA_URL, HA_TOKEN, CCU_URL ...) ausfuellen."
    exit 1
fi

# Sicherheits-Check: env-Datei darf nicht world-readable sein
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

# Publish-Guard: explizit AUS fuer Smoke-Test (Audit R-HIGH-3).
unset INVENTORY_PUBLISH || true
unset INVENTORY_PUBLISH_CONFIRM || true

# Sandbox-Pfade anlegen
mkdir -p "$(dirname "${INVENTORY_DB:-./local/inventory.db}")"
mkdir -p "${INVENTORY_YAML_DIR:-./local/yaml}"

echo "=== 1/4: DB-Migrationen ==="
"$BIN" migrate
echo

RC_HA=0
RC_CCU=0
RC_NODERED=0
DID_ANY=0

if [ -n "${HA_URL:-}" ] && [ -n "${HA_TOKEN:-}" ]; then
    DID_ANY=1
    echo "=== 2/4: Home Assistant Sync ($HA_URL) ==="
    if "$BIN" sync ha; then
        echo "  -> HA sync OK"
    else
        RC_HA=$?
        echo "  -> HA sync FAILED (exit $RC_HA)"
    fi
    echo
else
    echo "=== 2/4: HA Sync SKIPPED (HA_URL oder HA_TOKEN leer) ==="
    echo
fi

if [ -n "${CCU_URL:-}" ]; then
    DID_ANY=1
    echo "=== 3/4: CCU Sync ($CCU_URL) ==="
    if "$BIN" sync ccu; then
        echo "  -> CCU sync OK"
    else
        RC_CCU=$?
        echo "  -> CCU sync FAILED (exit $RC_CCU)"
    fi
    echo
else
    echo "=== 3/4: CCU Sync SKIPPED (CCU_URL leer) ==="
    echo
fi

if [ -n "${HA_URL:-}" ] && [ -n "${HA_TOKEN:-}" ] && [ -n "${NODERED_INGRESS_PATH:-}" ]; then
    DID_ANY=1
    echo "=== 4/4: Node-RED Sync (${HA_URL}/${NODERED_INGRESS_PATH}/flows) ==="
    if "$BIN" sync nodered; then
        echo "  -> Node-RED sync OK"
    else
        RC_NODERED=$?
        echo "  -> Node-RED sync FAILED (exit $RC_NODERED)"
    fi
    echo
else
    echo "=== 4/4: Node-RED Sync SKIPPED (HA_URL/HA_TOKEN/NODERED_INGRESS_PATH unvollstaendig) ==="
    echo
fi

if [ "$DID_ANY" = "0" ]; then
    echo "ERROR: weder HA noch CCU noch Node-RED konfiguriert — nichts zu testen."
    echo "  -> $ENV_FILE: mindestens HA_URL+HA_TOKEN oder CCU_URL setzen."
    exit 1
fi

echo "=== Ergebnis-Artefakte ==="
echo "  DB:   ${INVENTORY_DB}"
echo "  YAML: ${INVENTORY_YAML_DIR}/"
ls -la "${INVENTORY_YAML_DIR}/" 2>/dev/null || true
echo
if [ "$RC_HA" -ne 0 ] || [ "$RC_CCU" -ne 0 ] || [ "$RC_NODERED" -ne 0 ]; then
    echo "Mindestens ein Sync hat versagt — siehe Output oben."
    exit 3
fi
echo "Smoke-Test gruen."
