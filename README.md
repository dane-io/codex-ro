# About codex-ro

Run Codex inside a VM with project files mounted as read-only.

# Install
## Install incus and configure user groups
```bash
# Install incus, start the socket, and add user to incus-admin group so sudo isn't required
sudo dnf install incus
sudo systemctl enable --now incus.socket
sudo usermod -aG incus-admin "$USER"

# Logout / reboot, then check if user got added to incus-admin
id -nG
```

## Install codex-ro binary and initialize
```bash
cargo install

codex-ro config --init
```

## Configure firewalld settings (example using Fedora)

```bash
# Check which zone your normal Internet connection actually uses
sudo firewall-cmd --get-active-zones

# Create a dedicated firewalld zone for the Codex VM bridge
sudo firewall-cmd --permanent --new-zone=codex-vm

# Drop VM traffic aimed at the Fedora host unless explicitly allowed below
sudo firewall-cmd --permanent --zone=codex-vm --set-target=DROP

# Assign the Incus bridge to the new zone
sudo firewall-cmd --permanent --zone=codex-vm --change-interface=codexbr0

# Allow the VM to use the Incus DHCP server
sudo firewall-cmd --permanent --zone=codex-vm --add-port=67/udp

# Allow the VM to use the Incus DNS server
sudo firewall-cmd --permanent --zone=codex-vm --add-port=53/udp
sudo firewall-cmd --permanent --zone=codex-vm --add-port=53/tcp

# Create a policy controlling traffic forwarded from the Codex VM toward other networks
sudo firewall-cmd --permanent --new-policy=codex-egress

# Traffic entering this policy comes from the Codex VM zone
sudo firewall-cmd --permanent --policy=codex-egress --add-ingress-zone=codex-vm

# Permit the VM to leave only through the regular FedoraWorkstation egress zone. Other distros may need a different zone or ANY.
sudo firewall-cmd --permanent --policy=codex-egress --add-egress-zone=FedoraWorkstation

# Block RFC1918 private networks
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="10.0.0.0/8" reject'
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="172.16.0.0/12" reject'
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="192.168.0.0/16" reject'

# Block CGNAT/Tailscale address space
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="100.64.0.0/10" reject'

# Block IPv4 link-local addresses
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="169.254.0.0/16" reject'

# Block loopback address space from being used as a forwarded destination
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="127.0.0.0/8" reject'

# Block IPv4 multicast destinations
sudo firewall-cmd --permanent --policy=codex-egress --add-rich-rule='rule family="ipv4" destination address="224.0.0.0/4" reject'

# Allow all other forwarded IPv4 traffic, which gives the VM normal public Internet access
sudo firewall-cmd --permanent --policy=codex-egress --set-target=ACCEPT

# Apply the permanent configuration
sudo firewall-cmd --reload

# Verify that codexbr0 is assigned to the expected zone
sudo firewall-cmd --get-active-zones

# Show the host-facing rules for the Codex VM bridge
sudo firewall-cmd --zone=codex-vm --list-all

# Show the VM egress policy and its blocked destination networks
sudo firewall-cmd --info-policy=codex-egress
```


# Uninstall
```bash
# Delete incus VM
incus stop codex-vm
incus delete codex-vm

# List trusted interfaces in firewalld
sudo firewall-cmd --zone=trusted --list-interfaces
sudo firewall-cmd --permanent --zone=trusted --list-interfaces

# Assuming codexbr0 is in trusted zone
sudo firewall-cmd --permanent --zone=trusted --remove-interface=codexbr0
sudo firewall-cmd --reload

# If that doesn't remove it, check through nmcli:
nmcli -g connection.zone connection show codexbr0
sudo nmcli connection modify codexbr0 connection.zone ""
sudo firewall-cmd --reload

# If incusbr0 or codexbr0 still persists in trusted zone, remove the entry from /etc/firewalld/zones/trusted.xml
# Delete the line that looks like: <interface name="incusbr0"/>
sudo firewall-cmd --check-config
sudo firewall-cmd --reload
sudo firewall-cmd --permanent --zone=trusted --list-interfaces

# Delete incus network settings
incus network delete codexbr0
incus network acl delete codex-vm-internet

# Verify incus stuff is gone
incus list
incus network list
incus network acl list
```


**NOTE: this project assumes you have uv on the PATH**

# Use
## Allow list
`codex-ro` uses a whitelist stored in `~/.config/codex-ro/config.toml` to determine if your project can be mounted. This helps prevent accidentally mounting /home which could contain SSH keys, etc. To configure the allow list, check out the `allow` subcommand:
```bash
codex-ro whitelist -h
```
An example to add the current working directory:
```bash
codex-ro allow --add .
```

## Initialize VM


## Login to Codex


## Edit global AGENTS.md


## Run codex
By calling the `run` subcommand, `codex-ro` will check if your current working directory is in the allow list. If it is, the directory will be mounted as read only to the VM for Codex to see.
```bash
codex-ro run
```
