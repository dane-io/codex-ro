# About codex-ro

Run Codex inside a read-only Podman container while preserving your Codex login and configuration.

# Install

## Add Python script to PATH
After cloning this repository, navigate to it and use the following bash to be able to call the Python script from PATH:
```bash
mkdir -p "$HOME/.local/bin"
ln -sfn "$(realpath codex_ro.py)" "$HOME/.local/bin/codex-ro"
```

## Fix SELinux issues
This project does **not** rely on disabling SELinux to work. To fix the inevitable SELinux errors that normally arise on Fedora, install the `selinux-policy-devel` package and then do the following to compile the SELinux policy for `codex_bwrap_t`:
```bash
make -f /usr/share/selinux/devel/Makefile codex_bwrap.pp
sudo semodule -i codex_bwrap.pp
```

To uninstall:
```bash
sudo semodule -r codex_bwrap
```

**NOTE: this project assumes you have uv on the PATH**

# Use
## Allow list
`codex-ro` uses a whitelist stored in `~/.config/codex-ro/config.json` to determine if your project can be mounted. This helps prevent accidentally mounting /home which could contain SSH keys, etc. To configure the allow list, check out the `allow` subcommand:
```bash
codex-ro allow -h
```
An example to add the current working directory:
```bash
codex-ro allow --add .
```

## Build container
To initially build the Codex container or update it, use the `build` subcommand:
```bash
codex-ro build
codex-ro build --update
```

## Login to Codex
By logging in, Codex will store your credentials in the persistent `codex-home` Podman volume:
```bash
codex-ro login
```

## Run codex
By calling the `run` subcommand, `codex-ro` will check if your current working directory is in the allow list. If it is, the directory will be mounted as read only to the Podman container for Codex to see.
```bash
codex-ro run
```
