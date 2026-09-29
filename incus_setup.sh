# Do all of this before
# sudo dnf install incus
# sudo systemctl enable --now firewalld incus.socket
# sudo usermod -aG incus-admin "$USER"

# Logout / reboot, then check if user got added to incus-admin
# id -nG

incus admin init --minimal

incus network create codexbr0 \
  ipv4.address=auto ipv4.nat=true \
  ipv6.address=none

incus network acl create vm-internet

incus network acl rule add vm-internet egress action=reject \
  destination=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,100.64.0.0/10,169.254.0.0/16,127.0.0.0/8

incus network acl rule add vm-internet egress action=allow \
  destination=0.0.0.0/0

incus network set codexbr0 security.acls=vm-internet


# Configure firewalld
# TODO: probably add firewalld rules instead of using trusted zone in the future
sudo firewall-cmd --permanent --zone=trusted --change-interface=codexbr0
sudo firewall-cmd --reload

# Start VM and get a shell
incus launch images:debian/13 debian-vm --vm --no-profiles --storage default --network codexbr0
# If already started:
incus start debian-vm
incus exec debian-vm -- bash

# Turn off VM
incus stop debian-vm