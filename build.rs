extern crate git2;
extern crate toml;

use git2::{Repository};
use toml::{Value, from_str};

fn main() {
    let repo = Repository::open(".").expect("Current folder is not a git repositry");
    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let hash = oid.as_bytes().iter().take(4).map(|byte| format!("{:x}", byte)).collect::<String>();
    println!("cargo:rustc-env=GIT_HASH={}", hash);

    let file_str = include_str!("Cargo.toml");
    let config: Value = from_str(file_str).unwrap();
    let version_str = config.as_table().unwrap()["package"].as_table().unwrap()["version"].as_str().unwrap();
    println!("cargo:rustc-env=CRATE_VERSION={}", version_str);
}
