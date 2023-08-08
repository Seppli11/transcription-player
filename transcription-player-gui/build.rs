use std::{env, fs, io, path::Path, process::Command};

use gtk::gio;

const BLUEPRINT_DIR: &str = "./blueprints";
const RESOURCE_DIR: &str = "./resources";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    return;

    compile_blueprints().expect("Couldn't compile blueprints");

    gio::compile_resources(
        "resources",
        "resources/resources.gresource.xml",
        "transcrible.gresource",
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    let schemas_dir = format!("{out_dir}/schemas");
    let schemas_dir = Path::new(&schemas_dir);
    fs::create_dir_all(&schemas_dir).unwrap();

    fs::copy(
        "resources/ninja.seppli.Transcrible.gschema.xml",
        &schemas_dir.join("ninja.seppli.Transcrible.gschema.xml"),
    )
    .unwrap();

    Command::new("glib-compile-schemas")
        .arg(format!("{out_dir}/schemas"))
        .status()
        .unwrap();

    println!(
        "cargo:rustc-env=GSETTINGS_SCHEMA_DIR={}",
        schemas_dir.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed=resources/ninja.seppli.Transcrible.gschema.xml");
}

/// Compiles the *.blp blueprint files with compile-blueprint. The output is written to resources
///
/// # Errors
///
/// This function will return an error if
/// - The BLUEPRINT_DIR directory couldn't be read or the compilation process failed
fn compile_blueprints() -> Result<(), io::Error> {
    let blueprint_files = fs::read_dir(BLUEPRINT_DIR)?;
    let blueprint_files: Vec<String> = blueprint_files
        .filter_map(|file_option| {
            file_option
                .map(|file| file.path().as_os_str().to_string_lossy().into_owned())
                .ok()
        })
        .collect();

    for blueprint in &blueprint_files {
        println!("cargo:rerun-if-changed={}", blueprint);
    }

    if blueprint_files.len() > 0 {
        Command::new("blueprint-compiler")
            .arg("batch-compile")
            .arg(RESOURCE_DIR)
            .arg(BLUEPRINT_DIR)
            .args(blueprint_files)
            .status()?;
    }
    Ok(())
}
