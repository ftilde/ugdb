extern crate git2;
extern crate toml;

use git2::Repository;
use toml::{from_str, Value};

fn main() {
    // Find git revision of current version, if possible
    let revision_str = if let Ok(repo) = Repository::open(".") {
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        oid.as_bytes()
            .iter()
            .take(4)
            .map(|byte| format!("{:0length$x}", byte, length = 2))
            .collect::<String>()
    } else {
        " release".to_owned()
    };
    println!("cargo:rustc-env=REVISION={}", revision_str);

    // Find current release version (crate version specified in Cargo.toml)
    let file_str = include_str!("Cargo.toml");
    let config: Value = from_str(file_str).unwrap();
    let version_str = config.as_table().unwrap()["package"].as_table().unwrap()["version"]
        .as_str()
        .unwrap();
    println!("cargo:rustc-env=CRATE_VERSION={}", version_str);
}
