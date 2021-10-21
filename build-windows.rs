use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use build_target::Profile;
use walkdir::WalkDir;

const RUNNER_CRATE_NAME: &str = "virtual-desktop-runner";

pub fn main() {
    dbg!(env::vars().collect::<Vec<_>>());
    cargo_emit::rerun_if_changed!("{}", source_runner_crate_path().display());

    build_runner_inplace()
        .or_else(|_| build_runner_in_out_dir())
        .or_else(|_| build_runner_in_temp_dir())
        .unwrap();
}

fn build_runner_at_target(runner_crate_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--target")
        .arg(build_target::target_triple().unwrap())
        .arg("-Z")
        .arg("unstable-options")
        .arg("--out-dir")
        .arg(out_dir().join(RUNNER_CRATE_NAME).join("out"))
        .current_dir(runner_crate_path);
    if Profile::current().unwrap() == Profile::Release {
        command.arg("--release");
    }

    command.spawn()?.wait()?.exit_ok()?;

    Ok(())
}

fn copy_runner_crate_to(target_crate_path: &Path) -> Result<(), Box<dyn Error>> {
    if target_crate_path.join("Cargo.toml").exists() {
        return Ok(());
    }

    fs::create_dir_all(target_crate_path).unwrap();

    let source_runner_crate_path = source_runner_crate_path();
    let gitignore_path = source_runner_crate_path.join(".gitignore");
    let gitignore_file = gitignore::File::new(&gitignore_path).ok();
    let gitignore_filter = move |path: &Path| {
        !gitignore_file
            .as_ref()
            .map(|f| f.is_excluded(path).unwrap())
            .unwrap_or(false)
    };

    for entry in WalkDir::new(&source_runner_crate_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| gitignore_filter(e.path()))
    {
        if entry.path_is_symlink() {
            continue;
        }

        let source_path = entry.path();
        let relative_path = source_path.strip_prefix(&source_runner_crate_path)?;
        let target_path = target_crate_path.join(relative_path);
        if entry.file_type().is_dir() {
            fs::create_dir(&target_path).unwrap();
        } else {
            assert!(entry.file_type().is_file());
            fs::copy(&source_path, &target_path).unwrap();
        }
    }

    fs::copy(
        target_crate_path.join("Cargo.toml.why-no-bin-deps"),
        target_crate_path.join("Cargo.toml"),
    )
    .unwrap();

    Ok(())
}

fn build_runner_inplace() -> Result<(), Box<dyn Error>> {
    println!("build_runner_inplace");
    let runner_crate_real_config = source_runner_crate_path().join("Cargo.toml");
    if !runner_crate_real_config.exists() {
        return Err("Inplace Cargo.toml not present.".into());
    }
    build_runner_at_target(&source_runner_crate_path())
}

fn build_runner_in_out_dir() -> Result<(), Box<dyn Error>> {
    println!("build_runner_in_out_dir");
    let target_runner_crate_path = out_dir().join(RUNNER_CRATE_NAME).join("crate");
    copy_runner_crate_to(&target_runner_crate_path).unwrap();
    build_runner_at_target(&target_runner_crate_path)
}

fn build_runner_in_temp_dir() -> Result<(), Box<dyn Error>> {
    println!("build_runner_in_temp_dir");
    let target_runner_crate_dir = tempfile::Builder::new()
        .prefix(&format!("{}-", RUNNER_CRATE_NAME))
        .tempdir()
        .unwrap();
    let target_runner_crate_path = target_runner_crate_dir.path();

    copy_runner_crate_to(&target_runner_crate_path).unwrap();
    build_runner_at_target(&target_runner_crate_path)
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
        .canonicalize()
        .unwrap()
}

fn current_dir() -> PathBuf {
    env::current_dir()
        .unwrap()
        .canonicalize()
        .unwrap()
        .to_path_buf()
}

fn source_runner_crate_path() -> PathBuf {
    current_dir()
        .join(RUNNER_CRATE_NAME)
        .canonicalize()
        .unwrap()
}
