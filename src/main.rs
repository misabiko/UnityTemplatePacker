extern crate tar;
extern crate flate2;
extern crate fs_extra;

//use std::env;
use std::{fs, fs::File};
use flate2::Compression;
use flate2::write::GzEncoder;
use fs_extra::dir::CopyOptions;
use flate2::read::GzDecoder;
use tar::Archive;

fn main() {
	unpack_unity_template("C:\\Program Files\\Unity\\Hub\\Editor\\2019.3.3f1\\Editor\\Data\\Resources\\PackageManager\\ProjectTemplates\\com.unity.template.3d-4.2.6.tgz");

	archive_package();

	clean_directory();
}

fn unpack_unity_template(path: &str) {
	let tgz = File::open(path)
		.expect("Couldn't read the sample tgz.");
	let tar = GzDecoder::new(tgz);
	let mut archive = Archive::new(tar);
	archive.unpack(".")
		.expect("Couldn't unpack the sample tgz.");
}

fn clone_directory(path: &str) {
	let mut from_paths = Vec::new();
	from_paths.push(path.clone());

	fs_extra::copy_items(&from_paths, ".", &CopyOptions::new())
		.expect("Couldn't clone the package directory.");
}

fn archive_package() {
	let tgz = File::create("com.misabiko.template.clean-urp.tgz").unwrap();
	let enc = GzEncoder::new(tgz, Compression::default());
	let mut tar = tar::Builder::new(enc);
	tar.append_dir_all("package", "package");
}

fn clean_directory() {
	let mut from_paths = Vec::new();
	from_paths.push("package");

	fs_extra::remove_items(&from_paths)
		.expect("Couldn't remove the cloned package directory.");
}