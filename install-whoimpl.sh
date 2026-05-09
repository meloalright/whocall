#!/bin/sh
set -e

REPO="meloalright/who-ast"
BIN="whoimpl"
INSTALL_DIR="/usr/local/bin"

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/'
}

get_target() {
  OS=$(uname -s)
  ARCH=$(uname -m)

  case "${OS}-${ARCH}" in
    Darwin-arm64)  echo "aarch64-apple-darwin" ;;
    Darwin-x86_64) echo "x86_64-apple-darwin" ;;
    Linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
    Linux-aarch64) echo "aarch64-unknown-linux-gnu" ;;
    *) echo "Unsupported platform: ${OS}-${ARCH}" >&2; exit 1 ;;
  esac
}

VERSION=$(get_latest_version)
TARGET=$(get_target)
URL="https://github.com/${REPO}/releases/download/v${VERSION}/who-${TARGET}.tar.gz"

echo "Installing ${BIN} v${VERSION} (${TARGET})..."

TMP=$(mktemp -d)
curl -fsSL "${URL}" -o "${TMP}/who.tar.gz"
tar xzf "${TMP}/who.tar.gz" -C "${TMP}" "${BIN}"

if [ -w "${INSTALL_DIR}" ]; then
  mv "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
else
  sudo mv "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
fi

chmod +x "${INSTALL_DIR}/${BIN}"
rm -rf "${TMP}"

echo "${BIN} v${VERSION} installed to ${INSTALL_DIR}/${BIN}"
