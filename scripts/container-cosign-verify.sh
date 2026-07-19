#!/usr/bin/env bash
# C06 L56 / T-660 — consumer / deploy-side verify for GHCR sharecli images.
# Usage: bash scripts/container-cosign-verify.sh [image-ref-or-digest]
set -euo pipefail

IDENTITY_REGEXP="${COSIGN_IDENTITY_REGEXP:-https://github.com/KooshaPari/sharecli/.*}"
OIDC_ISSUER="${COSIGN_OIDC_ISSUER:-https://token.actions.githubusercontent.com}"

SUBJECT="${1:-}"
if [[ -z "${SUBJECT}" ]]; then
  if [[ -f sharecli-ci-image-digest.txt ]]; then
    SUBJECT="$(tr -d '[:space:]' <sharecli-ci-image-digest.txt)"
  else
    echo "usage: $0 <ghcr.io/.../sharecli:tag|@sha256:...>" >&2
    echo "or run from a CI workspace with sharecli-ci-image-digest.txt" >&2
    exit 2
  fi
fi

if [[ "${SUBJECT}" == "skipped-push" ]]; then
  echo "::error::digest file is skipped-push — no registry image to verify"
  exit 1
fi

command -v cosign >/dev/null 2>&1 || {
  echo "error: cosign required (https://docs.sigstore.dev/cosign/system_config/installation/)" >&2
  exit 1
}

echo "Verifying signature: ${SUBJECT}"
cosign verify "${SUBJECT}" \
  --certificate-identity-regexp "${IDENTITY_REGEXP}" \
  --certificate-oidc-issuer "${OIDC_ISSUER}"

echo "Verifying slsaprovenance attestation: ${SUBJECT}"
cosign verify-attestation --type slsaprovenance "${SUBJECT}" \
  --certificate-identity-regexp "${IDENTITY_REGEXP}" \
  --certificate-oidc-issuer "${OIDC_ISSUER}" >/dev/null

echo "OK — signature + attestation verified for ${SUBJECT}"
