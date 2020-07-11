extern crate tar;
extern crate flate2;
extern crate fs_extra;
extern crate serde_json;
extern crate serde_yaml;

use std::{
	fs::{self, File},
	path::Path,
	io,
	error::Error,
};
use flate2::{
	Compression,
	write::GzEncoder,
	read::GzDecoder,
};
use tar::Archive;
use fs_extra::dir::CopyOptions;
use serde_json::json;

mod config;

pub use config::Config;
use crate::config::PackerConfig;

fn unpack_unity_template<P: AsRef<Path>>(path: P) {
	let tgz = File::open(path)
		.expect("Couldn't read the sample tgz.");
	let tar = GzDecoder::new(tgz);
	let mut archive = Archive::new(tar);
	archive.unpack(".")
		.expect("Couldn't unpack the sample tgz.");
}

fn clone_directory<P: AsRef<Path>>(from: P, to: P) -> fs_extra::error::Result<u64> {
	let mut from_paths = Vec::new();
	from_paths.push(from);

	fs_extra::copy_items(&from_paths, to, &CopyOptions::new())
}

fn clone_directories(project_path: &Path) -> fs_extra::error::Result<()> {
	let project_data_path = Path::new("package/ProjectData~");

	clone_directory(project_path.join("Assets"), project_data_path.to_path_buf())?;
	clone_directory(project_path.join("Packages"), project_data_path.to_path_buf())?;
	clone_directory(project_path.join("ProjectSettings"), project_data_path.to_path_buf())?;

	Ok(())
}

fn archive_package() {
	let tgz = File::create("com.misabiko.template.clean-urp.tgz").unwrap();
	let enc = GzEncoder::new(tgz, Compression::default());
	let mut tar = tar::Builder::new(enc);
	tar.append_dir_all("package", "package").unwrap();
}

fn clean_directory() {
	let mut from_paths = Vec::new();
	from_paths.push("package");

	fs_extra::remove_items(&from_paths)
		.expect("Couldn't remove the cloned package directory.");
}

fn generate_template(project_path: &Path) -> io::Result<()> {
	match clone_directories(project_path) {
		Ok(result) => Ok(result),
		Err(e) => Err(io::Error::new(io::ErrorKind::NotFound, e.to_string()))
	}?;

	fs::remove_file(Path::new("package/ProjectData~").join("ProjectSettings").join("ProjectVersion.txt"))?;

	edit_package_json()?;

	edit_project_settings()?;

	archive_package();
	Ok(())
}

fn edit_package_json() -> io::Result<()> {
	let package_data = File::open("package/package.json")?;
	let mut parsed_package: serde_json::Value = serde_json::from_reader(package_data)?;

	parsed_package["name"] = json!("com.misabiko.template.clean-urp");
	parsed_package["displayName"] = json!("Clean URP");
	parsed_package["version"] = json!("0.1.0");
	parsed_package["description"] = json!("This is an empty 3D project that uses Unity's Universal Render Pipeline");

	fs::write("package/package.json", parsed_package.to_string())?;

	Ok(())
}

fn edit_project_settings() -> io::Result<()> {
	let f = File::open("package/ProjectData~/ProjectSettings/ProjectSettings.asset")?;
	let mut parsed_settings: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();

	let player_settings = parsed_settings.get_mut("PlayerSettings").unwrap().as_mapping_mut().unwrap();
	player_settings.insert(serde_yaml::Value::from("templatePackageId"), serde_yaml::Value::from("com.misabiko.template.clean-urp@0.1.0"));
	player_settings.insert(serde_yaml::Value::from("templateDefaultScene"), serde_yaml::Value::from("Assets/Scenes/MainScene.unity"));

	fs::write("package/ProjectData~/ProjectSettings/ProjectSettings.asset", serde_yaml::to_string(&parsed_settings).unwrap())?;
	Ok(())
}

pub fn run_cli(config: PackerConfig) -> Result<(), Box<dyn Error>> {
	unpack_unity_template(config.editor.templates_path.join("com.unity.template.3d-4.2.8.tgz"));

	for entry in fs::read_dir("package/ProjectData~")? {
		let entry = entry?;
		fs::remove_dir_all(entry.path())?;
	}

	//If there's an error generating the template, delete it before ending.
	if let Err(e) = generate_template(&config.project.path) {
		clean_directory();
		return Err(Box::new(e));
	}

	clean_directory();

	println!("com.misabiko.template.clean-urp.tgz was created.");

	Ok(())
}

pub fn run_help() {
	let usages = vec![
		"<project_path> [editor_path]"
	];

	println!("Usage:");
	for usage in usages.iter() {
		println!("\tunity_template_packer {}", usage);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const SAMPLE_PROJECT_PATH: &'static str = "C:/Users/misabiko/Documents/Coding/UnityProjects/Clean URP";
	const SAMPLE_EDITOR_PATH: &'static str = "C:/Program Files/Unity/Hub/Editor/2019.4.1f1";

	#[test]
	fn invalid_editor_path() {
		let non_existing_path = UnityEditor::new("ksdjngkjsfgn".as_ref());
		let invalid_path = UnityEditor::new(".".as_ref());

		assert!(non_existing_path.is_err(), "Non-existing editor path wasn't detected.");
		assert!(invalid_path.is_err(), "Invalid editor path wasn't detected.");
	}

	#[test]
	fn invalid_project_path() {
		let project = UnityProject::new(".".as_ref());
		assert!(project.is_err(), "Wrong project path wasn't detected.");
	}

	#[test]
	fn valid_editor_path() {
		let editor = UnityEditor::new(SAMPLE_EDITOR_PATH.as_ref());
		assert!(editor.is_ok());
	}

	#[test]
	fn valid_project_path() {
		let project = UnityProject::new(SAMPLE_PROJECT_PATH.as_ref());
		assert!(project.is_ok());
	}

	#[test]
	fn new_template_added() {
		let args = [
			String::from(""),
			String::from(SAMPLE_PROJECT_PATH),
			String::from(SAMPLE_EDITOR_PATH),
			String::from("Clean URP"),
			String::from("0.0.1"),
		];
		let config = Config::new(&args).unwrap();

		run_cli(config);
		assert!(PathBuf::from("com.misabiko.template.clean-urp.tgz").exists());
	}
}