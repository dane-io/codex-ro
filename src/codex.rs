use crate::incus::{run_incus, VM_NAME};
use std::io;


pub const CODEX_USER: &str = "codex";


pub fn create_codex_user() -> io::Result<()> {
    run_incus(&[
        "exec",
        VM_NAME,
        "--",
        "useradd",
        "--create-home",
        "--shell",
        "/bin/bash",
        CODEX_USER,
    ])?;

    run_incus(&[
        "exec",
        VM_NAME,
        "--",
        "install",
        "-d",
        "-o",
        CODEX_USER,
        "-g",
        CODEX_USER,
        &format!("/home/{CODEX_USER}/.codex"),
    ])
}


pub fn install_codex() -> io::Result<()> {
    run_incus(&["exec", VM_NAME, "--", "apt-get", "update"])?;

    run_incus(&[
        "exec", VM_NAME, "--",
        "env", "DEBIAN_FRONTEND=noninteractive",
        "apt-get", "install", "-y", "--no-install-recommends",
        "ca-certificates", "nodejs", "npm", "git", "unattended-upgrades",
    ])?;

    run_incus(&[
        "exec", VM_NAME, "--",
        "npm", "install", "--global", "@openai/codex",
    ])?;

    run_incus(&[
        "exec", VM_NAME, "--",
        "su", "--login", CODEX_USER,
        "--command", "codex --version",
    ])
}


pub fn run_codex() -> io::Result<()> {
    run_incus(&[
        "exec", VM_NAME, "--mode", "interactive", "--",
        "su", "--login", CODEX_USER,
        "--command",
        "exec codex --cd /workspace --sandbox read-only --ask-for-approval never --config 'web_search=\"disabled\"'",
    ])
}
