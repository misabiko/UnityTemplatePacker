extern crate tar;
extern crate flate2;
extern crate fs_extra;

use std::{fs, fs::File};
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use tar::Archive;
use fs_extra::dir::CopyOptions;
use std::io::{Result, Error, ErrorKind};

fn main() -> Result<()> {
	let project_path = Path::new("C:/Users/misabiko/Documents/Coding/UnityProjects/Clean URP");
	let project_data_path= Path::new("package/ProjectData~");

	unpack_unity_template("C:\\Program Files\\Unity\\Hub\\Editor\\2019.3.5f1\\Editor\\Data\\Resources\\PackageManager\\ProjectTemplates\\com.unity.template.3d-4.2.6.tgz");

	for entry in fs::read_dir(project_data_path)? {
		let entry = entry?;
		fs::remove_dir_all(entry.path())?;
	}

	//If there's an error generating the template, delete it before ending.
	if let Err(e) = generate_template(project_path, project_data_path) {
		clean_directory();
		return Err(e);
	}

	clean_directory();

	Ok(())
}

fn unpack_unity_template(path: &str) {
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

fn clone_directories(project_path: &Path, project_data_path: &Path) -> fs_extra::error::Result<()> {
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

fn generate_template(project_path: &Path, project_data_path: &Path) -> Result<()> {
	match clone_directories(project_path, project_data_path) {
		Ok(result) => Ok(result),
		Err(e) => Err(Error::new(ErrorKind::NotFound, e.to_string()))
	}?;

	fs::remove_file(project_data_path.join("ProjectSettings").join("ProjectVersion.txt"))?;

	archive_package();
	Ok(())
}