use std::{io, process::Command};
use crate::config::is_project_path_allowed;
use crate::agents::copy_agents_to_vm;
use crate::codex::{create_codex_user, install_codex, CODEX_USER};


pub const VM_NAME: &str = "codex-vm";
const VM_NIC: &str = "codexbr0";
const VM_ACL: &str = "codex-vm-internet";
const VM_IMAGE: &str = "images:debian/13";


pub fn run_incus(args: &[&str]) -> io::Result<()> {
    let status = Command::new("incus").args(args).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("incus exited with {status}")))
    }
}


fn init_incus() -> io::Result<()> {
    run_incus(&["admin", "init", "--minimal"])
}


fn create_bridge() -> io::Result<()> {
    run_incus(&[
        "network", "create", VM_NIC,
        "ipv4.address=auto",
        "ipv4.nat=true",
        "ipv4.firewall=false",
        "ipv6.address=none",
    ])
}


fn create_acl() -> io::Result<()> {
    run_incus(&["network", "acl", "create", VM_ACL])?;

    run_incus(&[
        "network", "acl", "rule", "add", VM_ACL, "egress",
        "action=reject",
        "destination=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,100.64.0.0/10,169.254.0.0/16,127.0.0.0/8",
    ])?;

    run_incus(&[
        "network", "acl", "rule", "add", VM_ACL, "egress",
        "action=allow",
        "destination=0.0.0.0/0",
    ])?;

    run_incus(&[
        "network", "set", VM_NIC,
        &format!("security.acls={}", VM_ACL),
    ])
}


fn create_vm() -> io::Result<()> {
    run_incus(&[
        "init", VM_IMAGE, VM_NAME,
        "--vm", "--no-profiles",
        "--storage", "default",
        "--network", VM_NIC,
    ])
}


fn wait_for_vm() -> io::Result<()> {
    run_incus(&[
        "wait", VM_NAME, "agent",
        "--timeout", "120",
    ])
}


fn start_vm_process() -> io::Result<()> {
    run_incus(&["start", VM_NAME])
}


fn vm_shell() -> io::Result<()> {
    run_incus(&[
        "exec",
        VM_NAME,
        "--mode",
        "interactive",
        "--",
        "su",
        "--login",
        CODEX_USER,
    ])
}


pub fn stop_vm() -> io::Result<()> {
    run_incus(&["stop", VM_NAME])
}


fn mount_project(project: &str) -> io::Result<()> {
    run_incus(&[
        "config", "device", "add",
        VM_NAME,
        "project",
        "disk",
        &format!("source={project}"),
        "path=/workspace",
        "readonly=true",
    ])
}


fn unmount_project() -> io::Result<()> {
    run_incus(&[
        "config", "device", "remove",
        VM_NAME,
        "project",
    ])
}


pub fn run_vm_session(project: &str) -> io::Result<()> {
    if !is_project_path_allowed(project)? {
        return Err(io::Error::other(
            "project path is not in the whitelist"
        ));
    }
    
    
    let project = std::fs::canonicalize(project)?;
    let project = project.to_str().ok_or_else(|| io::Error::other("project path is not valid UTF-8"))?;
    mount_project(project)?;

    if let Err(start_error) = start_vm_process() {
        let unmount_result = unmount_project();

        return match unmount_result {
            Ok(()) => Err(start_error),
            Err(unmount_error) => Err(io::Error::other(format!(
                "failed to start VM: {start_error}; additionally failed to unmount project: {unmount_error}"
            ))),
        };
    }

    let session_result = (|| -> io::Result<()> {
        wait_for_vm()?;
        copy_agents_to_vm()?;
        vm_shell()?;

        Ok(())
    })();

    let stop_result = stop_vm();
    let unmount_result = unmount_project();

    match (session_result, stop_result, unmount_result) {
        (Ok(()), Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(()), Ok(())) => Err(error),
        (Ok(()), Err(error), Ok(())) => Err(error),
        (Ok(()), Ok(()), Err(error)) => Err(error),

        (session, stop, unmount) => Err(io::Error::other(format!(
            "session={session:?}; stop={stop:?}; unmount={unmount:?}"
        ))),
    }
}


pub fn uninstall() -> io::Result<()> {
    let _ = run_incus(&["stop", VM_NAME]);  // May fail if VM is already stopped
    run_incus(&["delete", VM_NAME])?;
    run_incus(&["network", "delete", VM_NIC])?;
    run_incus(&["network", "acl", "delete", VM_ACL])
}


pub fn config_incus() -> io::Result<()> {
    init_incus()?;
    create_bridge()?;
    create_acl()?;
    create_vm()?;

    start_vm_process()?;
    let setup_result = (|| -> io::Result<()> {
        wait_for_vm()?;
        create_codex_user()?;
        install_codex()
    })();
    let stop_result = stop_vm();

    match (setup_result, stop_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(setup_error), Err(stop_error)) => Err(io::Error::other(format!(
            "VM setup failed: {setup_error}; additionally failed to stop VM: {stop_error}"
        ))),
    }
}