# About codex-ro

Run Codex inside a VM with project files mounted as read-only.

# Install
```bash
sudo dnf install incus
sudo systemctl enable --now incus.socket
sudo usermod -aG incus-admin "$USER"

# Logout / reboot, then check if user got added to incus-admin
id -nG
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
