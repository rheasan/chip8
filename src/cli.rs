use clap::{Arg, ArgAction, Command};

pub enum Chip8Command {
	Emulate {
		src: String,
	}
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
			.about("assemble a chip8 program. the input should be a plaintext assembly file")
			.arg(
				Arg::new("src")
				.help("source for the chip8 program")
				.num_args(1)
				.required(true)
				.action(ArgAction::Set)
			)
		)
		.get_matches();
	
	match matched.subcommand() {
		Some(("emulate", emulate_args)) => {
			if !emulate_args.args_present() {
				return None;
			}

			let src = emulate_args.get_one::<String>("src")?.to_owned();
			return Some(Chip8Command::Emulate { src });
		},
		_ => unreachable!()	
	}
}