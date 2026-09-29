mod incus;
use incus::{config_incus, start_vm, stop_vm, run_vm_session, uninstall};
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
            Command::new("start")
                .about("Start VM")
        )
        .subcommand(
            Command::new("stop")
                .about("Stop VM")
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


fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("run", args)) => {
            let project = args.get_one::<String>("project").expect("has default");
            println!("Would run Codex for {project}");
            _ = run_vm_session();
        }
        Some(("start", _args)) => {
            _ = start_vm();
        }
        Some(("stop", _args)) => {
            _ = stop_vm();
        }
        Some(("config", args)) => {
            if args.get_flag("check") {
                println!("Would check Incus configuration");
            }
            else if args.get_flag("init") {
                _ = config_incus();
            }
            else if args.get_flag("deinit") {
                _ = uninstall();
            }
            else if args.get_flag("login") {
                println!("Would log in to Codex");
            }
        }
        Some(("completions", args)) => {
            let shell = *args.get_one::<Shell>("shell").expect("required");
            let mut cli = build_cli();
            print_completions(shell, &mut cli);
        }
        _ => unreachable!("a subcommand is required"),
    }
}
