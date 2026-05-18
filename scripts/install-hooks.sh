#!/bin/sh
# Installiert die Repo-Hooks. Einmalig nach dem Klonen ausführen.
#
# Setzt core.hooksPath auf scripts/hooks — kein Kopieren/Symlinken nötig,
# die Hooks bleiben damit versioniert und für alle Klone identisch.
set -eu

repo_root=$(git rev-parse --show-toplevel)
git -C "$repo_root" config core.hooksPath scripts/hooks
chmod +x "$repo_root"/scripts/hooks/* 2>/dev/null || true

echo "core.hooksPath -> scripts/hooks gesetzt."
echo "Aktive Hooks:"
ls -1 "$repo_root/scripts/hooks"
