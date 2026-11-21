#!/usr/bin/env bash
# asscify installer — Linux / macOS
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.sh | bash -s -- --version v0.1.0
set -euo pipefail

REPO="flaxodotdev/asscify"
BIN="asscify"
INSTALL_DIR="${ASSCIFY_INSTALL_DIR:-${HOME}/.local/bin}"
VERSION="${ASSCIFY_VERSION:-latest}"

# --- args ---
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v) VERSION="$2"; shift 2 ;;
    --dir) INSTALL_DIR="$2"; shift 2 ;;
    --help|-h)
      cat <<EOF
asscify installer

Options:
  --version, -v <tag>   Release tag (default: latest)
  --dir <path>          Install directory (default: ~/.local/bin)
  ASSCIFY_INSTALL_DIR   Same as --dir
  ASSCIFY_VERSION       Same as --version
EOF
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

# --- detect platform ---
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux*)  OS="linux" ;;
  darwin*) OS="macos" ;;
  msys*|cygwin*|mingw*)
    echo "Windows detected. Use PowerShell:" >&2
    echo "  irm https://raw.githubusercontent.com/${REPO}/master/install.ps1 | iex" >&2
    exit 1
    ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "unsupported arch: $ARCH" >&2; exit 1 ;;
esac

TARGET="${OS}-${ARCH}"
ASSET="${BIN}-${TARGET}.tar.gz"

echo "==> asscify installer"
echo "    target : ${TARGET}"
echo "    version: ${VERSION}"
echo "    dir    : ${INSTALL_DIR}"

# --- resolve download URL ---
if [[ "$VERSION" == "latest" ]]; then
  API="https://api.github.com/repos/${REPO}/releases/latest"
  if command -v curl >/dev/null 2>&1; then
    TAG="$(curl -fsSL "$API" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
  else
    TAG="$(wget -qO- "$API" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
  fi
  if [[ -z "${TAG}" ]]; then
    echo "error: could not resolve latest release. Is the repo public with a release?" >&2
    echo "fallback: cargo install asscify --git https://github.com/${REPO}" >&2
    exit 1
  fi
  VERSION="$TAG"
fi

BASE="https://github.com/${REPO}/releases/download/${VERSION}"
URL="${BASE}/${ASSET}"
CHECKSUM_URL="${BASE}/checksums.txt"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> downloading ${ASSET}"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL --retry 3 -o "${TMP}/${ASSET}" "$URL"
  curl -fsSL --retry 3 -o "${TMP}/checksums.txt" "$CHECKSUM_URL" 2>/dev/null || true
else
  wget -q -O "${TMP}/${ASSET}" "$URL"
  wget -q -O "${TMP}/checksums.txt" "$CHECKSUM_URL" 2>/dev/null || true
fi

# --- optional checksum ---
if [[ -f "${TMP}/checksums.txt" ]] && command -v shasum >/dev/null 2>&1; then
  EXPECTED="$(grep -E "  ${ASSET}$|  \\./${ASSET}$|\\s${ASSET}$" "${TMP}/checksums.txt" | awk '{print $1}' | head -n1 || true)"
  if [[ -n "${EXPECTED}" ]]; then
    ACTUAL="$(shasum -a 256 "${TMP}/${ASSET}" | awk '{print $1}')"
    if [[ "$EXPECTED" != "$ACTUAL" ]]; then
      echo "error: checksum mismatch for ${ASSET}" >&2
      echo "  expected: ${EXPECTED}" >&2
      echo "  actual  : ${ACTUAL}" >&2
      exit 1
    fi
    echo "==> checksum ok"
  fi
elif [[ -f "${TMP}/checksums.txt" ]] && command -v sha256sum >/dev/null 2>&1; then
  EXPECTED="$(grep -E "  ${ASSET}$|\\s${ASSET}$" "${TMP}/checksums.txt" | awk '{print $1}' | head -n1 || true)"
  if [[ -n "${EXPECTED}" ]]; then
    ACTUAL="$(sha256sum "${TMP}/${ASSET}" | awk '{print $1}')"
    if [[ "$EXPECTED" != "$ACTUAL" ]]; then
      echo "error: checksum mismatch for ${ASSET}" >&2
      exit 1
    fi
    echo "==> checksum ok"
  fi
fi

# --- extract + install ---
tar -xzf "${TMP}/${ASSET}" -C "$TMP"
mkdir -p "$INSTALL_DIR"

if [[ -f "${TMP}/${BIN}" ]]; then
  SRC="${TMP}/${BIN}"
elif [[ -f "${TMP}/${BIN}-${TARGET}/${BIN}" ]]; then
  SRC="${TMP}/${BIN}-${TARGET}/${BIN}"
else
  SRC="$(find "$TMP" -type f -name "$BIN" | head -n1)"
fi

if [[ -z "${SRC}" || ! -f "${SRC}" ]]; then
  echo "error: binary not found inside archive" >&2
  exit 1
fi

install -m 755 "$SRC" "${INSTALL_DIR}/${BIN}"

# --- PATH hint ---
echo
echo "==> installed ${INSTALL_DIR}/${BIN}"
if ! command -v "$BIN" >/dev/null 2>&1; then
  case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      echo
      echo "Add to your shell profile:"
      echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
      ;;
  esac
fi

echo
"${INSTALL_DIR}/${BIN}" --help | head -n 8 || true
echo
echo "Run:  asscify"
echo "Help: asscify  then press ?"
