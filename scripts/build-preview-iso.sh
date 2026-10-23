#!/usr/bin/env bash
set -Eeuo pipefail
umask 022

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/artifacts"
WORK_DIR="${ROOT_DIR}/.iso-work"
RESULT_DIR="${WORK_DIR}/result"
KICKSTART="${ROOT_DIR}/kickstart/deku-minimal-live.ks"
PROFILE="${ROOT_DIR}/../deku-profiles/profiles/minimal-x86_64/profile.json"

if [[ "$(id -u)" -ne 0 ]]; then
  echo "ERROR: this experimental builder must run as root inside an isolated disposable build VM/container." >&2
  exit 2
fi
for tool in livemedia-creator lorax rpm sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || { echo "ERROR: required tool missing: $tool" >&2; exit 2; }
done
[[ -f "$KICKSTART" ]] || { echo "ERROR: Kickstart not found: $KICKSTART" >&2; exit 2; }
[[ -f "$PROFILE" ]] || { echo "ERROR: profile not found: $PROFILE" >&2; exit 2; }

rm -rf "$RESULT_DIR"
mkdir -p "$OUT_DIR" "$RESULT_DIR"
rm -f "$OUT_DIR"/*.iso "$OUT_DIR"/*.sha256 "$OUT_DIR"/*.txt "$OUT_DIR"/*.log

{
  echo "Deku Linux technology-preview ISO build"
  echo "status=experimental-not-a-supported-release"
  echo "bootstrap=fedora-44"
  echo "architecture=x86_64"
  echo "firmware_target=UEFI-first"
  echo "git_sha=${GITHUB_SHA:-unknown}"
  echo "workflow_run_id=${GITHUB_RUN_ID:-manual}"
  echo "builder_fedora_release=$(rpm -E %fedora)"
  echo "lorax_version=$(rpm -q --qf '%{VERSION}-%{RELEASE}\n' lorax 2>/dev/null || true)"
  echo "kickstart_sha256=$(sha256sum "$KICKSTART" | cut -d' ' -f1)"
  echo "profile_sha256=$(sha256sum "$PROFILE" | cut -d' ' -f1)"
  echo
  echo "[builder package inventory]"
  rpm -qa --qf '%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}\n' | sort
} > "$OUT_DIR/build-manifest.txt"

set +e
livemedia-creator \
  --make-iso \
  --no-virt \
  --ks="$KICKSTART" \
  --resultdir="$RESULT_DIR" \
  --logfile="$OUT_DIR/livemedia-creator.log" \
  --project="Deku Linux Technology Preview" \
  --releasever=44 \
  --volid="DEKU-TP-0.1" \
  --kernel-args="console=ttyS0,115200n8 console=tty0" \
  --timeout=90 \
  --iso-only 2>&1 | tee "$OUT_DIR/build.log"
status=${PIPESTATUS[0]}
set -e
if [[ "$status" -ne 0 ]]; then
  echo "ERROR: livemedia-creator failed with exit code $status" >&2
  exit "$status"
fi

mapfile -t ISOS < <(find "$RESULT_DIR" -type f -name '*.iso' -size +0c -print)
if [[ "${#ISOS[@]}" -ne 1 ]]; then
  printf 'ERROR: expected exactly one non-empty ISO, found %s\n' "${#ISOS[@]}" >&2
  find "$RESULT_DIR" -maxdepth 4 -type f -printf '%p (%s bytes)\n' >&2 || true
  exit 3
fi

OUTPUT_ISO="$OUT_DIR/deku-linux-technology-preview-x86_64.iso"
cp -- "${ISOS[0]}" "$OUTPUT_ISO"
sha256sum "$OUTPUT_ISO" > "$OUTPUT_ISO.sha256"
printf 'iso_bytes=%s\niso_sha256=%s\n' \
  "$(stat -c %s "$OUTPUT_ISO")" \
  "$(cut -d' ' -f1 "$OUTPUT_ISO.sha256")" >> "$OUT_DIR/build-manifest.txt"

echo "ISO created: $OUTPUT_ISO"
echo "WARNING: experimental Fedora-based technology preview; not a supported release."
