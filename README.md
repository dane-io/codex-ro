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


## Install codex-ro binary and autocompletions
```bash
# Navigate to the repo's folder. Add --force if needing to update
cargo install --path .

# Generate and save its completions (relies on bash-completion package)
mkdir -p ~/.local/share/bash-completion/completions
codex-ro completions bash > ~/.local/share/bash-completion/completions/codex-ro.bash

# Load them in this terminal
source ~/.local/share/bash-completion/completions/codex-ro.bash

codex-ro config --init
codex-ro agents --init
```


## Test firewalld settings
```bash
# Try pinging a remote URL inside the VM
# This can be done with codex-ro run --vm

# Temporarily disable firewalld policy
sudo firewall-cmd --policy=codex-egress --add-disable

# Try pining again

# Reload the firewalld policy and check the interface is back
sudo firewall-cmd --reload
sudo firewall-cmd --get-zone-of-interface=codexbr0
sudo firewall-cmd --zone=codex-vm --list-all
sudo firewall-cmd --info-policy=codex-egress
```


# Uninstall
```bash
codex-ro config --deinit

# Remove firewalld changes
sudo firewall-cmd --permanent --delete-policy=codex-egress
sudo firewall-cmd --permanent --delete-zone=codex-vm
sudo firewall-cmd --reload

cargo uninstall codex-ro
rm ~/.local/share/bash-completion/completions/codex-ro.bash

# Verify incus stuff is gone
incus list
incus network list
incus network acl list
```

# Use
## Allow list
`codex-ro` uses a whitelist stored in `~/.config/codex-ro/config.toml` to determine if your project can be mounted. This helps prevent accidentally mounting /home which could contain SSH keys, etc. To configure the allow list, check out the `whitelist` subcommand:
```bash
codex-ro whitelist -h
```
An example to add the current working directory:
```bash
codex-ro whitelist --add .
```

## Login to Codex
```bash
codex-ro config --login
```

## Edit global AGENTS.md
When the VM is started, `AGENTS.md` stored in `~/.config/codex-ro/AGENTS.md` will be copied into `~/.codex/AGENTS.md` inside the VM. To edit the copy saved outside the VM:
```bash
codex-ro agents --edit
```

## Run codex
By calling the `run` subcommand, `codex-ro` will check if your current working directory is in the allow list. If it is, the directory will be mounted as read only to the VM for Codex to see.
```bash
codex-ro run
```

Note, only one instance of `codex-ro` can be running at a time so duplicate entries aren't populated in `/workspace` inside the VM. This is done via the `vm.lock` file in `~/.config/codex-ro/vm.lock`
