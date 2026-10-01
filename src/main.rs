mod incus;
mod config;
mod agents;

use incus::{config_incus, run_vm_session, uninstall};
use config::{add_to_allowlist, remove_from_allowlist, list_allowlist};
use agents::{create_default_agents, edit_agents};
use clap::{Arg, ArgGroup, Command, ValueHint, ArgAction, value_parser};
use clap_complete::{Generator, Shell, generate};
use std::io;


fn build_cli() -> Command {
    Command::new("codex-ro")
        .about("Run Codex in an Incus VM")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run Codex for a project")
                .arg(
                    Arg::new("project")
                        .long("project")
                        .help("Project directory")
                        .default_value(".")
                        .value_hint(ValueHint::DirPath),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Configure codex-ro tool")
                .arg(
                    Arg::new("check")
                        .long("check")
                        .help("Check if config is good")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("init")
                        .long("init")
                        .help("Set up incus VMs and networking")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("deinit")
                        .long("deinit")
                        .help("Delete incus VMs, NICs, and ACLs")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("login")
                        .long("login")
                        .help("Login to Codex")
                        .action(ArgAction::SetTrue),
                )
                .group(
                    ArgGroup::new("config_action")
                        .args(["check", "init", "deinit", "login"])
                        .required(true),
                )
        )
        .subcommand(
            Command::new("whitelist")
                .about("Configure filepath whitelist at ~/.config/codex-ro/config.toml")
                .arg(
                    Arg::new("add")
                        .long("add")
                        .help("Add filepath to whitelist")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("remove")
                        .long("remove")
                        .help("Remove filepath from whitelist")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("list")
                        .long("list")
                        .help("List filepath from whitelist")
                        .action(ArgAction::SetTrue),
                )
                .group(
                    ArgGroup::new("config_action")
                        .args(["add", "remove", "list"])
                        .required(true),
                )
        )
        .subcommand(
            Command::new("agents")
                .about("Configure global AGENTS.md at ~/.config/codex-ro/AGENTS.md")
                .arg(
                    Arg::new("init")
                        .long("init")
                        .help("Copy default AGENTS.md")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("edit")
                        .long("edit")
                        .help("Edit AGENTS.md")
                        .action(ArgAction::SetTrue),
                )
        )
        .subcommand(
            Command::new("completions")
                .about("Print a shell completion script")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_parser(value_parser!(Shell)),
                ),
        )
}


fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(generator, cmd, cmd.get_name().to_owned(), &mut io::stdout());
}


fn main() -> io::Result<()> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("run", args)) => {
            let project = args.get_one::<String>("project").expect("has default");
            run_vm_session(project)?;
        }
        Some(("config", args)) => {
            if args.get_flag("check") {
                println!("Would check Incus configuration");
            }
            else if args.get_flag("init") {
                config_incus()?;
            }
            else if args.get_flag("deinit") {
                uninstall()?;
            }
            else if args.get_flag("login") {
                println!("Would log in to Codex");
            }
        }
        Some(("whitelist", args)) => {
            if let Some(path) = args.get_one::<String>("add") {
                add_to_allowlist(path)?;
            }
            else if let Some(path) = args.get_one::<String>("remove") {
                remove_from_allowlist(path)?;
            }
            else if args.get_flag("list") {
                list_allowlist()?;
            }
        }
        Some(("agents", args)) => {
            if args.get_flag("init") {
                create_default_agents()?;
            }
            else if args.get_flag("edit") {
                edit_agents()?;
            }
        }
        Some(("completions", args)) => {
            let shell = *args.get_one::<Shell>("shell").expect("required");
            let mut cli = build_cli();
            print_completions(shell, &mut cli);
        }
        _ => unreachable!("a subcommand is required"),
    }

    Ok(())
}
