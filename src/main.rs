extern crate tar;
extern crate flate2;
extern crate fs_extra;
extern crate serde_json;
extern crate serde_yaml;

use std::{fs, fs::File};
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use tar::Archive;
use fs_extra::dir::CopyOptions;
use std::io::{Result, Error, ErrorKind};
use serde_json::json;
use std::env;

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();
	println!("{:?}", args);

	let project_path = Path::new(&args[1]);
	let editor_path = Path::new(&args[2]);

	unpack_unity_template(editor_path.join("Editor\\Data\\Resources\\PackageManager\\ProjectTemplates\\com.unity.template.3d-4.2.6.tgz"));

	for entry in fs::read_dir("package/ProjectData~")? {
		let entry = entry?;
		fs::remove_dir_all(entry.path())?;
	}

	//If there's an error generating the template, delete it before ending.
	if let Err(e) = generate_template(project_path) {
		clean_directory();
		return Err(e);
	}

	clean_directory();

	Ok(())
}

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

fn generate_template(project_path: &Path) -> Result<()> {
	match clone_directories(project_path) {
		Ok(result) => Ok(result),
		Err(e) => Err(Error::new(ErrorKind::NotFound, e.to_string()))
	}?;

	fs::remove_file(Path::new("package/ProjectData~").join("ProjectSettings").join("ProjectVersion.txt"))?;

	edit_package_json()?;

	edit_project_settings()?;

	archive_package();
	Ok(())
}

fn edit_package_json() -> Result<()> {
	let package_data = File::open("package/package.json")?;
	let mut parsed_package: serde_json::Value = serde_json::from_reader(package_data)?;

	parsed_package["name"] = json!("com.misabiko.template.clean-urp");
	parsed_package["displayName"] = json!("Clean URP");
	parsed_package["version"] = json!("0.1.0");
	parsed_package["description"] = json!("This is an empty 3D project that uses Unity's Universal Render Pipeline");

	fs::write("package/package.json", parsed_package.to_string())?;

	Ok(())
}

fn edit_project_settings() -> Result<()> {
	let f = File::open("package/ProjectData~/ProjectSettings/ProjectSettings.asset")?;
	let mut parsed_settings: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();

	let player_settings = parsed_settings.get_mut("PlayerSettings").unwrap().as_mapping_mut().unwrap();
	player_settings.insert(serde_yaml::Value::from("templatePackageId"), serde_yaml::Value::from("com.misabiko.template.clean-urp@0.1.0"));
	player_settings.insert(serde_yaml::Value::from("templateDefaultScene"), serde_yaml::Value::from("Assets/Scenes/MainScene.unity"));

	fs::write("package/ProjectData~/ProjectSettings/ProjectSettings.asset", serde_yaml::to_string(&parsed_settings).unwrap())?;
	Ok(())
}