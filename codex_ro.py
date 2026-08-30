#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

import json
import os
from pathlib import Path
import argparse


APP_NAME = "codex-ro"

STATE_VOLUME = "codex-home"

DEFAULT_CONFIG = {
    "allowed_paths": [],
    "image": "localhost/codex-ro",
}


def fail(message):
    raise SystemExit(f"{APP_NAME}: {message}")


def config_path():
    return Path.home() / ".config" / APP_NAME / "config.json"


def load_config():
    path = config_path()

    if not path.exists():
        return DEFAULT_CONFIG.copy()

    try:
        loaded = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        fail(f"could not read {path}: {error}")

    if not isinstance(loaded, dict):
        fail(f"{path} must contain a JSON object")

    config = {**DEFAULT_CONFIG, **loaded}   # Load default first and allow loaded to overwrite

    if not isinstance(config["allowed_paths"], list):
        fail("'allowed_paths' must be a list")

    if not all(isinstance(path, str) for path in config["allowed_paths"]):
        fail("every allowed path must be a string")

    return config


def save_config(config):
    path = config_path()
    path.parent.mkdir(parents=True, exist_ok=True)

    temporary_path = path.with_suffix(".tmp")
    temporary_path.write_text(json.dumps(config, indent=2) + "\n")
    temporary_path.replace(path)


def canonical_path(value):
    return str(Path(value).expanduser().resolve())


def exec_command(command):
    try:
        os.execvp(command[0], command)
    except FileNotFoundError:
        fail(f"command not found: {command[0]}")


def list_allowlist():
    config = load_config()

    if not config["allowed_paths"]:
        print("No allowed paths.")

    else:
        for path in config["allowed_paths"]:
            print(path)
    return


def add_to_allowlist(path: str):
    config = load_config()
    path = canonical_path(path)

    if path not in config["allowed_paths"]:
        config["allowed_paths"].append(path)
        print(f"Added: {path}")

    save_config(config)


def remove_from_allowlist(path: str):
    config = load_config()
    path = canonical_path(path)

    if path in config["allowed_paths"]:
        config["allowed_paths"].remove(path)
        print(f"Removed: {path}")
    else:
        print(f"Not allowed: {path}")

    save_config(config)


# Return --security-opt option and bool if SELinux relabeling needs added
def host_security_options() -> tuple[str, bool]:
    # Check for SELinux
    if Path("/sys/fs/selinux/enforce").exists():
        return ("--security-opt=label=type:codex_bwrap_t", True)

    apparmor_enabled_path = Path("/sys/module/apparmor/parameters/enabled")
    if apparmor_enabled_path.exists():
        try:
            if apparmor_enabled_path.read_text().strip().lower() == "y":
                return ("--security-opt=apparmor=codex-ro", False)
        except OSError:
            pass

    fail("Neither SELinux nor AppArmor is enabled!")



def build_container(force_update: bool):
    config = load_config()
    project_root = Path(__file__).resolve().parent

    if not (project_root / "Containerfile").exists():
        fail(f"Containerfile not found next to {Path(__file__).name}")

    build_options = []
    if force_update:
        build_options.extend([
            "--pull=always",
            "--no-cache",
        ])

    exec_command(
        [
            "podman",
            "build",
            *build_options,
            "-t",
            config["image"],
            str(project_root),
        ]
    )


def login_to_codex():
    config = load_config()
    security_options, selinux_relabel = host_security_options()
    volume_options = ":z" if selinux_relabel else ""

    exec_command(
        [
            "podman",
            "run",
            "--rm",
            "-it",
            "--network",
            "host",
            security_options,
            f"--volume={STATE_VOLUME}:/root/.codex{volume_options}",
            "--entrypoint",
            "codex",
            config["image"],
            "login",
        ]
    )


def run_codex():
    config = load_config()
    workspace = canonical_path(Path.cwd())
    allowed_paths = [
        canonical_path(path) for path in config["allowed_paths"]
    ]

    if workspace not in allowed_paths:
        suggested_command = f"codex-ro allow --add {workspace}"
        fail(
            f"refusing to run from unapproved folder: {workspace}\n"
            f"To allow it, run: {suggested_command}"
        )

    security_options, selinux_relabel = host_security_options()
    workspace_options = ":ro,z" if selinux_relabel else ":ro"
    volume_options = ":z" if selinux_relabel else ""

    exec_command(
        [
            "podman",
            "run",
            "--rm",
            "-it",
            "--read-only",
            security_options,
            f"--volume={workspace}:/workspace{workspace_options}",
            f"--volume={STATE_VOLUME}:/root/.codex{volume_options}",
            "--workdir",
            "/workspace",
            config["image"],
        ]
    )


def main():
    if os.geteuid() == 0:
        fail("refusing to run as root")

    parser = argparse.ArgumentParser(
                    prog='codex-ro',
                    description='Run Codex as read only in isolated Podman container with SELinux / AppArmor support.')
    subcommands = parser.add_subparsers(dest="command", required=True)
    run_cmd = subcommands.add_parser("run", help="Run Codex")
    allow_cmd = subcommands.add_parser("allow", help="Configure allow list")
    allow_cmd.add_argument("--add", type=str, metavar="PATH", help="Add path to allow list")
    allow_cmd.add_argument("--remove", type=str, metavar="PATH", help="Remove path from allow list")
    allow_cmd.add_argument("--list", action="store_true", default=False, help="List all paths in allow list")
    build_cmd = subcommands.add_parser("build", help="Build Podman container")
    build_cmd.add_argument("--update", action="store_true", default=False, help="Build container without cache to force update")
    login_cmd = subcommands.add_parser("login", help="Sign in to Codex")

    args = parser.parse_args()
    
    match args.command:
        case "run":
            run_codex()

        case "allow":
            if args.add:
                add_to_allowlist(args.add)
            
            if args.remove:
                remove_from_allowlist(args.remove)

            if args.list:
                list_allowlist()

        case "build":
            build_container(args.update)

        case "login":
            login_to_codex()
        
        


if __name__ == "__main__":
    main()