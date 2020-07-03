mod gui;

use std::env;
use std::process;

use unity_template_packer::Config;
use crate::gui::TemplatePacker;


fn main() {
	//project_path, editor_path, template_name, template_version
	let args: Vec<String> = env::args().collect();

	/*let config = Config::new(&args).unwrap_or_else(|err| {
		eprintln!("Problem parsing arguments: {}", err);
		process::exit(1);
	});*/

	if let Ok(config) = Config::new(&args) {
		if let Err(err) = unity_template_packer::run_cli(config) {
			eprintln!("Application error: {}", err);
			process::exit(1);
		}
	}else {
		TemplatePacker::run_default();
	}
}