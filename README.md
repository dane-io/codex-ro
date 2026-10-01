# About codex-ro

Run Codex inside a VM with project files mounted as read-only.

# Install
```bash
# Install incus, start the socket, and add user to incus-admin group so sudo isn't required
sudo dnf install incus
sudo systemctl enable --now incus.socket
sudo usermod -aG incus-admin "$USER"

# Logout / reboot, then check if user got added to incus-admin
id -nG

# Change firewalld settings
sudo firewall-cmd --permanent --zone=trusted --change-interface=codexbr0
sudo firewall-cmd --reload
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
