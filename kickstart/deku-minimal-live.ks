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
livesys-scripts
anaconda
anaconda-live
anaconda-install-env-deps
grub2-efi-x64
grub2-efi-x64-cdboot
grub2-efi-x64-modules
grub2-pc
grub2-pc-modules
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

cat > /etc/systemd/system/deku-boot-marker.service <<'EOF'
[Unit]
Description=Deku Linux disposable VM boot acceptance marker
After=systemd-user-sessions.service
ConditionPathExists=/dev/ttyS0

[Service]
Type=oneshot
StandardOutput=tty
TTYPath=/dev/ttyS0
TTYReset=yes
TTYVHangup=yes
ExecStart=/usr/bin/printf DEKU_BOOT_OK:minimal-x86_64\\n

[Install]
WantedBy=multi-user.target
EOF
systemctl enable deku-boot-marker.service
rm -f /var/lib/systemd/random-seed
rm -f /etc/machine-id
touch /etc/machine-id
%end
