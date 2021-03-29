use anyhow::Result;
use gumdrop::Options;
use log::LevelFilter;

#[derive(Debug, Options)]
pub struct CliOptions {
	#[options(help = "filepath to log to (if left empty the program doesn't log)")]
	log_file: Option<String>,

	#[options(count, help = "verbosity. Once - info, twice - debug")]
	verbose: usize,

	#[options(help = "show this text")]
	help: bool,
}

impl CliOptions {
	pub fn check() -> Result<()> {
		let opts = CliOptions::parse_args_default_or_exit();

		if opts.help {
			println!("{}", CliOptions::usage());
			return Ok(());
		}

		if let Some(file) = opts.log_file {
			let lvl = match opts.verbose {
				2 => LevelFilter::Debug,
				1 => LevelFilter::Info,
				_ => LevelFilter::Warn,
			};
			simple_logging::log_to_file(file, lvl).unwrap();
		}

		Ok(())
	}
}
