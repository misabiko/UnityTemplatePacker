extern crate tar;
extern crate flate2;
extern crate fs_extra;
extern crate serde_json;
extern crate serde_yaml;
extern crate iced;

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

use iced::{button, Button, Text, Column, Sandbox, Element, Settings, text_input, Container, Length};

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

fn pack(project_path: &Path, editor_path: &Path) -> Result<()> {
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

#[derive(Default)]
struct TemplatePacker {
	src_project_input: text_input::State,
	src_project_value: String,
	editor_input: text_input::State,
	editor_value: String,
	pack_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
	Pack,
	SrcProjectChanged(String),
	EditorChanged(String),
}

impl Sandbox for TemplatePacker {
	type Message = Message;

	fn new() -> Self {
		Self::default()
	}

	fn title(&self) -> String {
		String::from("Unity Template Packer")
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::Pack => println!("Pack!"),
			Message::SrcProjectChanged(value) => self.src_project_value = value,
			Message::EditorChanged(value) => self.editor_value = value,
		}
	}

	fn view(&mut self) -> Element<Message> {
		let column = Column::new()
			.spacing(20)
			.padding(20)
			.max_width(600)
			.push(Text::new("Source Project Path:"))
			.push(
				text_input::TextInput::new(
					&mut self.src_project_input,
					"Source Project Path",
					&self.src_project_value,
					Message::SrcProjectChanged
				)
				.padding(10)
				.size(20)
			)
			.push(Text::new("Editor Project Path:"))
			.push(
				text_input::TextInput::new(
					&mut self.editor_input,
					"Editor Path",
					&self.editor_value,
					Message::EditorChanged
				)
				.padding(10)
				.size(20)
			)
			.push(
				Button::new(&mut self.pack_button, Text::new("Pack"))
					.on_press(Message::Pack)
			);

		Container::new(column)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.into()
	}
}

fn parse_args(args: &Vec<String>) -> Result<(&Path, &Path)> {
	let project_path = Path::new(&args[1]);
	if !project_path.exists() {
		return Err(Error::new(ErrorKind::InvalidInput, format!("Project Path \"{}\" doesn't exist.", &args[1])));
	}
	let editor_path = Path::new(&args[2]);
	if !editor_path.exists() {
		return Err(Error::new(ErrorKind::InvalidInput, format!("Editor Path \"{}\" doesn't exist.", &args[1])));
	}

	Ok((project_path, editor_path))
}

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() > 1 {
		let (project_path, editor_path) = parse_args(&args)?;

		return pack(project_path, editor_path);
	}else {
		TemplatePacker::run(Settings::default());
	}

	Ok(())
}