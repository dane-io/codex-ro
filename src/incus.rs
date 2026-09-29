use std::{io, process::Command};


const VM_NAME: &str = "codex-vm";
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
        //"ipv4.firewall=false",
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


pub fn start_vm() -> io::Result<()> {
    run_incus(&["start", VM_NAME])
}


pub fn vm_shell() -> io::Result<()> {
    run_incus(&["exec", VM_NAME, "--", "bash"])
}


pub fn uninstall() -> io::Result<()> {
    let _ = run_incus(&["stop", VM_NAME]);  // May fail if VM is already stopped
    run_incus(&["delete", VM_NAME])?;
    run_incus(&["network", "delete", VM_NIC])?;
    run_incus(&["network", "acl", "delete", VM_ACL])
}


pub fn stop_vm() -> io::Result<()> {
    run_incus(&["stop", VM_NAME])
}


pub fn config_incus() -> io::Result<()> {
    init_incus()?;
    create_bridge()?;
    create_acl()?;
    create_vm()
}