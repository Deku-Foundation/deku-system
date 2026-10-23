# Deku Linux technology-preview live image.
# This is a Fedora 44-based prototype, not an independent Deku distribution.
# Do not use this image to install on a physical machine.

#version=DEVEL
lang en_US.UTF-8
keyboard --xlayouts=us
timezone Etc/UTC --utc
network --bootproto=dhcp --device=link --activate
firewall --enabled
selinux --enforcing
rootpw --lock
shutdown
bootloader --location=none
clearpart --all --initlabel
reqpart
part / --size=10240

url --url="https://download.fedoraproject.org/pub/fedora/linux/releases/44/Everything/x86_64/os/"
repo --name="updates" --baseurl="https://download.fedoraproject.org/pub/fedora/linux/updates/44/Everything/x86_64/"

services --disabled="sshd" --enabled="NetworkManager,livesys,livesys-late"

%packages
@core
systemd
bash
coreutils
util-linux
iproute
iputils
kernel
kernel-modules
kernel-modules-extra
dracut-network
dracut-config-generic
dracut-live
NetworkManager
grub2-pc
grub2-pc-modules
grub2-tools
grub2-tools-extra
livesys-scripts
anaconda
anaconda-live
anaconda-install-env-deps
grub2-efi-x64
grub2-efi-x64-cdboot
grub2-efi-x64-modules
shim-x64
efibootmgr
tar
%end

%post --erroronfail
set -e
cat > /etc/deku-release <<'EOF'
Deku Linux technology preview
Bootstrap: Fedora Linux 44
Profile: minimal-x86_64
Status: experimental; not an independent distribution
EOF

rpm -qa --qf '%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}\n' | LC_ALL=C sort > /etc/deku-package-manifest.txt

install -D -m 0755 /dev/stdin /usr/local/libexec/deku-boot-acceptance.sh <<'SCRIPT'
#!/usr/bin/bash
set -Eeuo pipefail

fail() {
  printf 'DEKU_BOOT_FAIL:%s\n' "$1"
  exit 1
}

check() {
  local name="$1"
  shift
  if "$@"; then
    printf 'DEKU_CHECK_OK:%s\n' "$name"
  else
    fail "$name"
  fi
}

check release-metadata test -s /etc/deku-release
check package-manifest test -s /etc/deku-package-manifest.txt
printf 'DEKU_PACKAGE_MANIFEST_BEGIN\n'
cat /etc/deku-package-manifest.txt
printf 'DEKU_PACKAGE_MANIFEST_END\n'
check network-manager systemctl is-active --quiet NetworkManager
if systemctl is-enabled --quiet sshd; then
  fail sshd-enabled
fi
printf 'DEKU_CHECK_OK:sshd-not-enabled\n'
if [[ -r /sys/fs/selinux/enforce && "$(cat /sys/fs/selinux/enforce)" == "1" ]]; then
  printf 'DEKU_CHECK_OK:selinux-enforcing\n'
else
  fail selinux-not-enforcing
fi
printf 'DEKU_BOOT_OK:minimal-x86_64\n'
SCRIPT

cat > /etc/systemd/system/deku-boot-marker.service <<'EOF'
[Unit]
Description=Deku Linux disposable VM boot acceptance checks
After=NetworkManager.service systemd-user-sessions.service
Wants=NetworkManager.service
ConditionPathExists=/dev/ttyS0

[Service]
Type=oneshot
StandardOutput=tty
TTYPath=/dev/ttyS0
TTYReset=yes
TTYVHangup=yes
ExecStart=/usr/local/libexec/deku-boot-acceptance.sh

[Install]
WantedBy=multi-user.target
EOF
systemctl enable deku-boot-marker.service
rm -f /var/lib/systemd/random-seed
rm -f /etc/machine-id
touch /etc/machine-id
%end
