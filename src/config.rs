use std::{io, fs};
use std::path::{PathBuf, Path};

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

pub struct PackerConfig {
	pub project: UnityProject,
	pub editor: UnityEditor,
}

pub enum Config {
	Packer (PackerConfig),
	GUI,
	Help,
}

fn list_editors() -> io::Result<Vec<(String, PathBuf)>> {
	let editors_path = "C:/Program Files/Unity/Hub/Editor";
	let mut editors = Vec::new();

	for entry in fs::read_dir(editors_path)? {
		if let Ok(entry) = entry {
			let path = entry.path();
			if path.is_dir() {
				if let Some(file_name) = path.file_name() {
					if let Some(file_name) = file_name.to_str() {
						editors.push((file_name.into(), path));
					}
				}
			}
		}
	}

	Ok(editors)
}

fn ask_editor() -> io::Result<UnityEditor> {
	let editors = list_editors()?;

	if editors.len() == 1 {
		return UnityEditor::new(&editors[0].1);
	}

	println!("Please choose an editor for the template:");
	for (i, entry) in editors.iter().enumerate() {
		println!("{}- {}", i, &entry.0)
	}

	let mut input = String::new();

	loop {
		if let Ok(_) = io::stdin().read_line(&mut input) {
			if let Ok(index) = input.trim_end().parse::<usize>() {
				if index < editors.len() {
					return UnityEditor::new(&editors[index].1)
				}
			}
		}

		println!("Please enter a number between 0 and {}.", editors.len() - 1);
		input.clear();
	};
}

impl Config {
	pub fn new(args: &[String]) -> io::Result<Config> {
		match args.len() {
			2 => {
				let project = UnityProject::new(args[1].as_ref())?;
				let editor = ask_editor()?;

				Ok(Config::Packer (PackerConfig {
					project,
					editor,
				}))
			},
			3 => {
				let project = UnityProject::new(args[1].as_ref())?;
				let editor = UnityEditor::new(args[2].as_ref())?;

				Ok(Config::Packer (PackerConfig {
					project,
					editor,
				}))
			},
			_ => Ok(Config::Help {}),
		}
	}
}