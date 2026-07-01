#!/bin/sh
set -eu

IMAGE_NAME="${IMAGE_NAME:-lls}"

VERSION="$(awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml)"
GIT_REVISION="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
BUILD_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

docker build \
  --build-arg GIT_REVISION="$GIT_REVISION" \
  --build-arg BUILD_DATE="$BUILD_DATE" \
  --build-arg VERSION="$VERSION" \
  -t "${IMAGE_NAME}:dev" \
  -t "${IMAGE_NAME}:${VERSION}" \
  -f Containerfile \
  .
