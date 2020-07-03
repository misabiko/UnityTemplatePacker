extern crate tar;
extern crate flate2;
extern crate fs_extra;
extern crate serde_json;
extern crate serde_yaml;

use std::{
	fs::{self, File},
	path::{Path, PathBuf},
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

fn pack(project_path: &Path, editor_path: &Path) -> io::Result<()> {
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

	println!("com.misabiko.template.clean-urp.tgz was created.");

	Ok(())
}

pub struct UnityEditor {
	pub path: PathBuf,
	pub templates_path: PathBuf,
}

impl UnityEditor {
	pub fn new(path: &Path) -> io::Result<Self> {
		if let Err(err) = UnityEditor::check_path(path) {
			return Err(err);
		}

		Ok(UnityEditor {
			path: PathBuf::from(path),
			templates_path: UnityEditor::get_template_path(path)?,
		})
	}

	fn check_path(path: &Path) -> io::Result<()> {
		if !path.exists() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Editor path is not valid: {:?}", path)));
		}

		Ok(())
	}

	fn get_template_path(path: &Path) -> io::Result<PathBuf> {
		let template_path = path.join("Editor\\Data\\Resources\\PackageManager\\ProjectTemplates");

		if !template_path.exists() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Editor's template path does not exist: {:?}", template_path)));
		}

		if !fs::read_dir(&template_path)?.any(
			|entry| entry.unwrap().path().extension().unwrap() == "tgz"
		) {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "Editor's template path does not contain a template"));
		}

		Ok(template_path)
	}
}

pub struct UnityProject {
	pub path: PathBuf,
}

impl UnityProject {
	pub fn new(path: &Path) -> io::Result<Self> {
		if let Err(err) = UnityProject::check_path(path) {
			return Err(err);
		}

		Ok(UnityProject {
			path: PathBuf::from(path)
		})
	}

	fn check_path(path: &Path) -> io::Result<()> {
		if !path.exists() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Project path does not exist: {:?}", path)));
		}

		let project_version_path = path.join("ProjectSettings/ProjectVersion.txt");
		if !project_version_path.exists() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Project path isn't a valid project, missing: {:?}", project_version_path)));
		}

		//TODO compare project version with editor version

		Ok(())
	}
}

pub struct Config {
	pub project: UnityProject,
	pub editor: UnityEditor,
	pub template_name: String,
	pub template_version: String,
}

impl Config {
	pub fn new(args: &[String]) -> io::Result<Config> {
		if args.len() < 5 {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "not enough arguments"));
		}

		let project = UnityProject::new(args[1].as_ref())?;
		let editor = UnityEditor::new(args[2].as_ref())?;
		let template_name = String::from(&args[3]);
		let template_version = String::from(&args[4]);

		Ok(Config { project, editor, template_name, template_version })
	}
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
	if let Some(template_path) = config.editor.templates_path.to_str() {
		println!("Template path: {}", template_path);
	}

	Ok(())
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

		let new_template_path = config.editor.templates_path.with_file_name(config.template_name.to_owned() + "-" + &config.template_version + ".tgz");
		println!("New Template Path: {:?}", new_template_path);

		run(config);
		assert!(new_template_path.exists());
	}
}