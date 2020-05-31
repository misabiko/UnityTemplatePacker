use std::env;
use std::process;

use unity_template_packer::Config;

fn main() {
	let args: Vec<String> = env::args().collect();

	let config = Config::new(&args).unwrap_or_else(|err| {
		eprintln!("Problem parsing arguments: {}", err);
		process::exit(1);
	});

	if let Err(err) = unity_template_packer::run(config) {
		//eprintln!("Application error: {}", err);
		eprintln!("Application error.");
		process::exit(1);
	}
}