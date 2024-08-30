use std::{
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

use transparent::{CommandExt, TransparentRunner};

#[test]
fn check_identical_output() {
    let test_text = "Test-ĄЂइ₡⍓☉あ句︽％";

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell.exe");
        c.arg("-Command")
            .arg(format!("Write-Output '{}'", test_text));
        c
    } else {
        let mut c = Command::new("echo");
        c.arg(test_text);
        c
    };
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let transparent_output = cmd
        .spawn_transparent(&TransparentRunner::new())
        .unwrap()
        .wait_with_output()
        .unwrap();
    let opaque_output = cmd.spawn().unwrap().wait_with_output().unwrap();

    assert_eq!(transparent_output.status, opaque_output.status);
    assert_eq!(transparent_output.stdout, opaque_output.stdout);
    assert_eq!(transparent_output.stderr, opaque_output.stderr);
}

#[test]
fn check_identical_output_specific_id() {
    let test_text = "Test-ĄЂइ₡⍓☉あ句︽％";

    let mut cmd = if cfg!(unix) {
        let mut c = Command::new("echo");
        c.arg(test_text);
        c
    } else {
        return;
    };
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let transparent_output = cmd
        .spawn_transparent(&TransparentRunner::with_args(
            99,
            "/tmp/123Xauth".to_owned(),
        ))
        .unwrap()
        .wait_with_output()
        .unwrap();
    let opaque_output = cmd.spawn().unwrap().wait_with_output().unwrap();

    assert_eq!(transparent_output.status, opaque_output.status);
    assert_eq!(transparent_output.stdout, opaque_output.stdout);
    assert_eq!(transparent_output.stderr, opaque_output.stderr);
}

#[test]
fn check_identical_non_zero_exit_code() {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell.exe");
        c.arg("-Command").arg("§"); // Invalid command to make PS fail
        c
    } else {
        Command::new("false")
    };
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let transparent_output = cmd
        .spawn_transparent(&TransparentRunner::new())
        .unwrap()
        .wait_with_output()
        .unwrap();
    let opaque_output = cmd.spawn().unwrap().wait_with_output().unwrap();

    assert!(!transparent_output.status.success());
    assert_eq!(transparent_output.status, opaque_output.status);
    assert_eq!(transparent_output.stdout, opaque_output.stdout);
    assert_eq!(transparent_output.stderr, opaque_output.stderr);
}
