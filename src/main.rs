mod gui;

use std::env;
use std::process;

use unity_template_packer::Config;
use crate::gui::TemplatePacker;


fn main() {
	let args: Vec<String> = env::args().collect();

	match Config::new(&args) {
		Ok(config) => match config {
			Config::Help => unity_template_packer::run_help(),
			Config::GUI => TemplatePacker::run_default(),
			Config::Packer(packer_config) => if let Err(err) = unity_template_packer::run_cli(packer_config) {
				eprintln!("Application error: {}", err);
				process::exit(1);
			},
		},
		Err(err) => {
			eprintln!("Problem parsing arguments: {}", err);
			process::exit(1);
		}
	}
}