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

impl Config {
	pub fn new(args: &[String]) -> io::Result<Config> {
		if args.len() < 3 {
			Ok(Config::Help {})
		}else {
			let project = UnityProject::new(args[1].as_ref())?;
			let editor = UnityEditor::new(args[2].as_ref())?;

			Ok(Config::Packer (PackerConfig {
				project,
				editor,
			}))
		}
	}
}