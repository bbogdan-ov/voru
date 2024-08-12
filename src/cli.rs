use std::path::PathBuf;

use crate::commands::Commands;

pub fn print_version() {
    println!("VORU v{} - {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_DESCRIPTION"));
    println!("{}", env!("CARGO_PKG_AUTHORS"));
}
pub fn print_help(commands: &Commands) {
    print_version();
    println!();
    println!("USAGE:");
    println!("    voru [OPTIONS] [COMMAND]");
    println!();
    println!("COMMANDS:");

    let cmd_list = commands.formatted_list();
    let mut max_name_width = 0_usize;

    for (is_alias, cmd_name, _) in &cmd_list {
        if *is_alias { continue; }
        max_name_width = max_name_width.max(cmd_name.len());
    }
    for (is_alias, cmd_name, cmd_desc) in &cmd_list {
        if *is_alias { continue; }
        let spaces = max_name_width + 2 - cmd_name.len();

        println!("    {}{}{}", cmd_name, " ".repeat(spaces), cmd_desc);
    }

    println!();
    println!("    help                 Print this message");
    println!("    version              Print current version");
    println!();
    println!("OPTIONS:");
    println!("    -v, --version        Print current version");
    println!("    -h, --help           Print this message again!");
    println!("    -c, --config <PATH>  Specify path to config.toml");
    println!("    --echo <MSG>         Send a command with a message");
    println!();
    println!("EXAMPLES:");
    println!("    Launch VORU with a welcome message!");
    println!("        voru --echo 'HELLO!!!'");
    println!();
    println!("    Shuffle the queue and send a message:");
    println!("        voru --echo 'Queue shuffled!' queue-shuffle");
    println!();
    println!("    Add tracks to the queue:");
    println!("        voru add ~/my-cool-music/*");
}

/// Cli
#[derive(Debug, Default)]
pub struct Cli {
    pub print_version: bool,
    pub print_help: bool,
    pub config_path: Option<PathBuf>,
    pub echo_msg: Option<String>
}
impl Cli {
    /// Tries to parse options and commands from a list of args
    /// Returns `None` if an unknown argument was given
    pub fn parse(args: &[String]) -> Option<Self> {
        let mut cli = Self::default();
        let mut args_iter = args.iter();

        loop {
            let Some(arg) = args_iter.next() else {
                break;
            };

            match arg.as_str() {
                "-h" | "--help" | "help" => {
                    cli.print_help = true;
                    break;
                }
                "-v" | "--version" | "version" => {
                    cli.print_version = true;
                    break;
                }
                "-c" | "--config" => {
                    cli.config_path = args_iter.next().map(|p| p.into());
                }
                "--echo" => {
                    cli.echo_msg = args_iter.next().cloned();
                }
                _ => return None
            }
        }

        Some(cli)
    }
}
