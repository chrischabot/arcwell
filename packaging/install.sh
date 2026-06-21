#!/usr/bin/env bash
set -euo pipefail

# CLAIM: A release archive is installed only after an exact SHA-256 match and a
# safe tar listing check; the installer replaces only the target binary and does
# not install or start services by default.
# PRECONDITIONS: The caller supplies either a local archive plus sha256 or a
# version/base-url pair with a checksums.txt file.
# ORACLE: SHA-256 digest equality, tar member validation, staged executable
# checks, and an atomic mv into the chosen prefix.
# SEVERITY: Severe supply-chain/path handling coverage when paired with
# scripts/verify-packaging-artifacts --self-test.

PREFIX="${PREFIX:-$HOME/.local}"
VERSION="${ARCWELL_VERSION:-}"
TARGET="${ARCWELL_TARGET:-auto}"
BASE_URL="${ARCWELL_RELEASE_BASE_URL:-https://github.com/chrischabot/arcwell/releases/download}"
ARCHIVE=""
EXPECTED_SHA256="${ARCWELL_SHA256:-}"
KEEP_TEMP=0

usage() {
  cat <<'USAGE'
Usage:
  packaging/install.sh --version vX.Y.Z [--target target] [--prefix dir] [--base-url url]
  packaging/install.sh --archive path --sha256 hex [--prefix dir]

Installs the arcwell binary from a checksummed release archive. This script is
non-destructive: it does not install, enable, or start launchd/systemd services.

Targets:
  aarch64-apple-darwin, x86_64-apple-darwin,
  aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu
USAGE
}

fail() {
  printf '[arcwell-install] FAIL: %s\n' "$*" >&2
  exit 1
}

log() {
  printf '[arcwell-install] %s\n' "$*"
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    fail "missing sha256sum or shasum"
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os:$arch" in
    Darwin:arm64) echo "aarch64-apple-darwin" ;;
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Linux:aarch64|Linux:arm64) echo "aarch64-unknown-linux-gnu" ;;
    Linux:x86_64|Linux:amd64) echo "x86_64-unknown-linux-gnu" ;;
    *) fail "unsupported target for uname ${os}/${arch}; pass --target explicitly" ;;
  esac
}

copy_or_download() {
  local src="$1"
  local dest="$2"
  if [[ "$src" == file://* ]]; then
    cp "${src#file://}" "$dest"
  elif [[ "$src" == http://* || "$src" == https://* ]]; then
    need_cmd curl
    curl --fail --location --show-error --silent "$src" --output "$dest"
  elif [[ -f "$src" ]]; then
    cp "$src" "$dest"
  else
    fail "unsupported or missing archive/checksum source: $src"
  fi
}

fetch_expected_sha() {
  local checksums="$1"
  local archive_name="$2"
  local checksum_file="$3"
  copy_or_download "$checksums" "$checksum_file"
  awk -v name="$archive_name" '
    $1 ~ /^[0-9a-fA-F]{64}$/ && $2 == name { print tolower($1); found=1 }
    END { if (!found) exit 1 }
  ' "$checksum_file" || fail "checksums file does not contain an exact entry for $archive_name"
}

validate_archive_listing() {
  local archive="$1"
  local saw_bin=0
  local mode first
  while IFS= read -r member; do
    [[ "$member" != *$'\n'* ]] || fail "unsafe archive member path contains newline"
    case "$member" in
      ""|/*|*"/../"*|../*|*".."|*"/..") fail "unsafe archive member path: $member" ;;
    esac
    [[ "$member" == "arcwell" || "$member" == "./arcwell" || "$member" == */arcwell ]] && saw_bin=1
  done < <(tar -tzf "$archive")
  while IFS= read -r listing; do
    mode="${listing%% *}"
    first="${mode:0:1}"
    case "$first" in
      -|d) ;;
      *) fail "unsupported archive member type in tar listing: $listing" ;;
    esac
  done < <(tar -tvzf "$archive")
  [[ "$saw_bin" == "1" ]] || fail "archive does not contain an arcwell binary"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --archive) ARCHIVE="${2:-}"; shift ;;
    --sha256) EXPECTED_SHA256="${2:-}"; shift ;;
    --version) VERSION="${2:-}"; shift ;;
    --target) TARGET="${2:-}"; shift ;;
    --prefix) PREFIX="${2:-}"; shift ;;
    --base-url) BASE_URL="${2:-}"; shift ;;
    --keep-temp) KEEP_TEMP=1 ;;
    -h|--help) usage; exit 0 ;;
    *) fail "unknown argument: $1" ;;
  esac
  shift
done

need_cmd tar
need_cmd awk
need_cmd mktemp

if [[ "$TARGET" == "auto" ]]; then
  TARGET="$(detect_target)"
fi

case "$TARGET" in
  aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu) ;;
  *) fail "unsupported target: $TARGET" ;;
esac

TMP="$(mktemp -d "${TMPDIR:-/tmp}/arcwell-install.XXXXXX")"
cleanup() {
  if [[ "$KEEP_TEMP" == "1" ]]; then
    log "kept temp dir: $TMP"
  else
    rm -rf "$TMP"
  fi
}
trap cleanup EXIT

if [[ -z "$ARCHIVE" ]]; then
  [[ -n "$VERSION" ]] || fail "--version is required when --archive is not supplied"
  archive_name="arcwell-${VERSION}-${TARGET}.tar.gz"
  release_url="${BASE_URL%/}/${VERSION}"
  ARCHIVE="$TMP/$archive_name"
  copy_or_download "$release_url/$archive_name" "$ARCHIVE"
  EXPECTED_SHA256="$(fetch_expected_sha "$release_url/checksums.txt" "$archive_name" "$TMP/checksums.txt")"
else
  [[ -f "$ARCHIVE" ]] || fail "archive does not exist: $ARCHIVE"
  archive_name="$(basename "$ARCHIVE")"
fi

[[ -n "$EXPECTED_SHA256" ]] || fail "--sha256 is required for local archives"
[[ "$EXPECTED_SHA256" =~ ^[0-9a-fA-F]{64}$ ]] || fail "expected sha256 must be 64 hex characters"

actual_sha256="$(sha256_file "$ARCHIVE")"
actual_sha256="$(printf '%s' "$actual_sha256" | tr 'A-F' 'a-f')"
EXPECTED_SHA256="$(printf '%s' "$EXPECTED_SHA256" | tr 'A-F' 'a-f')"
[[ "$actual_sha256" == "$EXPECTED_SHA256" ]] || fail "sha256 mismatch for $archive_name"

validate_archive_listing "$ARCHIVE"

extract_dir="$TMP/extract"
mkdir -p "$extract_dir"
tar -xzf "$ARCHIVE" -C "$extract_dir"
candidate="$(find "$extract_dir" -type f -name arcwell -perm -111 | head -n 1)"
[[ -n "$candidate" ]] || fail "extracted archive does not contain an executable arcwell binary"

install_dir="$PREFIX/bin"
mkdir -p "$install_dir"
staged="$install_dir/.arcwell.install.$$"
cp "$candidate" "$staged"
chmod 755 "$staged"
"$staged" --help >/dev/null || fail "staged arcwell binary failed --help"
mv "$staged" "$install_dir/arcwell"

log "installed $install_dir/arcwell from $archive_name"
log "services were not installed or started; run arcwell service install or scripts/install-systemd-user explicitly when ready"
