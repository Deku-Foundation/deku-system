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
for tool in livemedia-creator lorax rpm sha256sum python3; do
  command -v "$tool" >/dev/null 2>&1 || { echo "ERROR: required tool missing: $tool" >&2; exit 2; }
done
[[ -f "$KICKSTART" ]] || { echo "ERROR: Kickstart not found: $KICKSTART" >&2; exit 2; }
[[ -f "$PROFILE" ]] || { echo "ERROR: profile not found: $PROFILE" >&2; exit 2; }

python3 - "$PROFILE" "$KICKSTART" <<'PY'
import json
import re
import sys
from pathlib import Path

profile_path, kickstart_path = map(Path, sys.argv[1:])
profile = json.loads(profile_path.read_text(encoding="utf-8"))
kickstart = kickstart_path.read_text(encoding="utf-8")
if profile.get("schema_version") != 1:
    raise SystemExit("ERROR: unsupported profile schema_version")
if profile.get("target", {}).get("architecture") != "x86_64":
    raise SystemExit("ERROR: this builder only accepts the x86_64 profile")
if "uefi" not in profile.get("target", {}).get("firmware", []):
    raise SystemExit("ERROR: profile must declare UEFI support")
bootstrap = profile.get("bootstrap", {})
if bootstrap.get("distribution") != "fedora" or bootstrap.get("release") != "44":
    raise SystemExit("ERROR: builder only accepts the explicitly reviewed Fedora 44 bootstrap")
if bootstrap.get("independent_deku_rootfs") is not False:
    raise SystemExit("ERROR: technology-preview profile must not claim an independent Deku root filesystem")
match = re.search(r"(?ms)^%packages\s*\n(.*?)^%end\s*$", kickstart)
if not match:
    raise SystemExit("ERROR: Kickstart has no valid %packages section")
listed = {
    line.strip().split()[0]
    for line in match.group(1).splitlines()
    if line.strip() and not line.lstrip().startswith(("#", "@", "-"))
}
required = set(profile.get("package_selection", []))
missing = sorted(required - listed)
if missing:
    raise SystemExit("ERROR: profile packages missing from Kickstart: " + ", ".join(missing))
required_live = {
    "grub2-pc", "grub2-pc-modules", "grub2-tools", "grub2-tools-extra",
    "grub2-efi-x64", "grub2-efi-x64-cdboot", "grub2-efi-x64-modules",
    "efibootmgr", "dracut-live", "dracut-config-generic"
}
missing_live = sorted(required_live - listed)
if missing_live:
    raise SystemExit("ERROR: Kickstart omits required live-image/bootloader packages: " + ", ".join(missing_live))
if "After=multi-user.target" in kickstart:
    raise SystemExit("ERROR: boot marker service must not order itself after multi-user.target")
print(f"Validated profile {profile.get('profile_id')} with {len(required)} explicit package selections.")
PY

rm -rf "$RESULT_DIR"
mkdir -p "$OUT_DIR" "$WORK_DIR"
rm -f "$OUT_DIR"/*.iso "$OUT_DIR"/*.sha256 "$OUT_DIR"/*.txt "$OUT_DIR"/*.log

{
  echo "Deku Linux technology-preview ISO build"
  echo "status=experimental-not-a-supported-release"
  echo "bootstrap=fedora-44"
  echo "architecture=x86_64"
  echo "firmware_target=UEFI-first"
  echo "git_sha=${GITHUB_SHA:-unknown}"
  echo "workflow_run_id=${GITHUB_RUN_ID:-manual}"
  echo "builder_image=${BUILDER_IMAGE_DIGEST:-unknown}"
  echo "builder_fedora_release=$(rpm -E %fedora)"
  echo "lorax_version=$(rpm -q --qf '%{VERSION}-%{RELEASE}\n' lorax 2>/dev/null || true)"
  echo "kickstart_sha256=$(sha256sum "$KICKSTART" | cut -d' ' -f1)"
  echo "profile_sha256=$(sha256sum "$PROFILE" | cut -d' ' -f1)"
  echo "profile_commit=${PROFILE_SHA:-unknown}"
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
