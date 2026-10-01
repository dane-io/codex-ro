use std::{env, fs, io, path::PathBuf, process::Command};
use crate::incus::{run_incus, VM_NAME};


const DEFAULT_AGENTS: &str = 
r#"- The project is mounted read-only under /workspace.
- Restrict filesystem searches to the project directory.
- Do not attempt to modify project files or system files.
"#;


fn agents_path() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::other("HOME is not set"))?;

    Ok(PathBuf::from(home)
        .join(".config")
        .join("codex-ro")
        .join("AGENTS.md"))
}


pub fn create_default_agents() -> io::Result<()> {
    let path = agents_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, DEFAULT_AGENTS)?;

    println!("Created {}", path.display());

    Ok(())
}


pub fn edit_agents() -> io::Result<()> {
    let path = agents_path()?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, DEFAULT_AGENTS)?;
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(editor)
        .arg(&path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "editor exited with {status}"
        )))
    }
}


pub fn copy_agents_to_vm() -> io::Result<()> {
    let path = agents_path()?;
    if !path.exists() {
        return Ok(());
    }
    let path = path.to_str().ok_or_else(|| io::Error::other("AGENTS.md path is not valid UTF-8"))?;

    let destination = format!("{VM_NAME}/root/codex/.codex/AGENTS.md");

    run_incus(&["file", "push", "--create-dirs", path, &destination])
}
