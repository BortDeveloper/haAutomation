#!/usr/bin/env bash
# smoke-test.sh — Erster Verbindungstest gegen Test-HA + Test-CCU.
#
# Liest local/test-setup.env (siehe test-setup.env.example), checkt
# Pflichtwerte und fuehrt:
#   1. `inventory migrate`               (SQLite anlegen)
#   2. `inventory sync ha`               (sofern HA_URL+HA_TOKEN gesetzt)
#   3. `inventory sync ccu`              (sofern CCU_URL gesetzt)
#
# aus. Die Binary muss vorher gebaut sein:
#   cargo build --release --locked --bin inventory
#
# Exit-Codes:
#   0   alle aktivierten Quellen erfolgreich
#   1   Config-Fehler / fehlende Pflichtwerte
#   2   Binary nicht gefunden
#   3+  Sync-Fehler (HA oder CCU)

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

echo "=== 1/3: DB-Migrationen ==="
"$BIN" migrate
echo

RC_HA=0
RC_CCU=0
DID_ANY=0

if [ -n "${HA_URL:-}" ] && [ -n "${HA_TOKEN:-}" ]; then
    DID_ANY=1
    echo "=== 2/3: Home Assistant Sync ($HA_URL) ==="
    if "$BIN" sync ha; then
        echo "  -> HA sync OK"
    else
        RC_HA=$?
        echo "  -> HA sync FAILED (exit $RC_HA)"
    fi
    echo
else
    echo "=== 2/3: HA Sync SKIPPED (HA_URL oder HA_TOKEN leer) ==="
    echo
fi

if [ -n "${CCU_URL:-}" ]; then
    DID_ANY=1
    echo "=== 3/3: CCU Sync ($CCU_URL) ==="
    if "$BIN" sync ccu; then
        echo "  -> CCU sync OK"
    else
        RC_CCU=$?
        echo "  -> CCU sync FAILED (exit $RC_CCU)"
    fi
    echo
else
    echo "=== 3/3: CCU Sync SKIPPED (CCU_URL leer) ==="
    echo
fi

if [ "$DID_ANY" = "0" ]; then
    echo "ERROR: weder HA noch CCU konfiguriert — nichts zu testen."
    echo "  -> $ENV_FILE: mindestens HA_URL+HA_TOKEN oder CCU_URL setzen."
    exit 1
fi

echo "=== Ergebnis-Artefakte ==="
echo "  DB:   ${INVENTORY_DB}"
echo "  YAML: ${INVENTORY_YAML_DIR}/"
ls -la "${INVENTORY_YAML_DIR}/" 2>/dev/null || true
echo
if [ "$RC_HA" -ne 0 ] || [ "$RC_CCU" -ne 0 ]; then
    echo "Mindestens ein Sync hat versagt — siehe Output oben."
    exit 3
fi
echo "Smoke-Test gruen."
