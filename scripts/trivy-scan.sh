#!/usr/bin/env bash
# Lokales Trivy-Gate fuer das inventory-Image.
#
# Faehrt Trivy als Container (aquasec/trivy:latest) gegen ein lokal gebautes
# Image und bricht bei HIGH/CRITICAL-Findings mit Exit != 0 ab. Kein lokaler
# Trivy-Install noetig — nur Docker.
#
# Aufruf:
#   scripts/trivy-scan.sh                       # scannt inventory:dev
#   scripts/trivy-scan.sh inventory:1.2.3       # scannt das angegebene Image
#   SEVERITY=CRITICAL scripts/trivy-scan.sh     # nur CRITICAL
#
# Exit-Codes:
#   0 — keine Funde der gewaehlten Severities
#   1 — Funde gefunden (Gate failed)
#   2 — Aufruf-/Setup-Fehler

set -euo pipefail

IMAGE="${1:-inventory:dev}"
SEVERITY="${SEVERITY:-HIGH,CRITICAL}"
TRIVY_IMAGE="${TRIVY_IMAGE:-aquasec/trivy:latest}"
TRIVY_CACHE_DIR="${TRIVY_CACHE_DIR:-${HOME}/.cache/trivy}"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker im PATH erforderlich." >&2
  exit 2
fi

# Pruefe, dass das Ziel-Image lokal existiert — Trivy koennte es zwar selber
# pullen, aber das versteckt fehlende Builds. Hier wollen wir das lokal
# gebaute Image scannen.
if ! docker image inspect "${IMAGE}" >/dev/null 2>&1; then
  echo "ERROR: Image '${IMAGE}' nicht lokal vorhanden. Erst 'docker build' laufen lassen." >&2
  exit 2
fi

mkdir -p "${TRIVY_CACHE_DIR}"

echo "Trivy-Scan: image=${IMAGE} severity=${SEVERITY}"

docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  -v "${TRIVY_CACHE_DIR}:/root/.cache/" \
  "${TRIVY_IMAGE}" \
  image \
    --severity "${SEVERITY}" \
    --exit-code 1 \
    --ignore-unfixed \
    --no-progress \
    "${IMAGE}"
