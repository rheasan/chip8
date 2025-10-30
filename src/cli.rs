use clap::{Arg, ArgAction, Command};

pub enum Chip8Command {
    Emulate {
        src: String,
        debug: bool,
        timing: bool,
    },
    PrintKeyMap,
}

pub fn parse_args() -> Option<Chip8Command> {
    let matched = Command::new("chip8")
        .about("a chip8 emulator")
        .author("rheasan :3")
        .arg_required_else_help(true)
        .subcommand_required(true)
        // emulate
        .subcommand(
            Command::new("emulate")
                .about("assemble and run a chip8 program. the input should be a binary c8 assembly file")
                .arg(
                    Arg::new("src")
                        .help("source for the chip8 program")
                        .num_args(1)
                        .required(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("debug")
                        .help("print extra debug logs")
                        .long("debug")
                        .short('d')
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
                .arg(
                    Arg::new("timing")
                        .help("print time taken per consecutive instructions")
                        .long("timing")
                        .short('t')
                        .action(ArgAction::SetTrue)
                        .required(false)
                ),
        )
        .subcommand(Command::new("keymap").about("print keymap"))
        .get_matches();

    match matched.subcommand() {
        Some(("emulate", emulate_args)) => {
            if !emulate_args.args_present() {
                return None;
            }

            let src = emulate_args.get_one::<String>("src")?.to_owned();
            let debug = *emulate_args.get_one::<bool>("debug").unwrap_or(&false);
            let timing = *emulate_args.get_one::<bool>("timing").unwrap_or(&false);
            return Some(Chip8Command::Emulate { src, debug, timing });
        }
        Some(("keymap", _)) => return Some(Chip8Command::PrintKeyMap),
        _ => unreachable!(),
    }
}
