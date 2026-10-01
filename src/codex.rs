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
