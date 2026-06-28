use assert_cmd::Command;
use std::fs;

#[test]
fn generate_default_size() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("test.ndjson");
    let output_str = output.to_str().unwrap();

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("1KB")
        .arg("--output")
        .arg(output_str)
        .assert()
        .success();
}

#[test]
fn generate_creates_valid_ndjson() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("valid.ndjson");
    let output_str = output.to_str().unwrap();

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("2KB")
        .arg("--output")
        .arg(output_str)
        .assert()
        .success();

    let contents = fs::read_to_string(&output).unwrap();
    for line in contents.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "invalid JSON: {line}"
        );
    }
}

#[test]
fn generate_file_size_approximately_correct() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("sized.ndjson");
    let output_str = output.to_str().unwrap();

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("10KB")
        .arg("--output")
        .arg(output_str)
        .assert()
        .success();

    let size = fs::metadata(&output).unwrap().len();
    assert!(size >= 10 * 1024, "file too small: {size} bytes");
    assert!(size < 12 * 1024, "file too large: {size} bytes");
}

#[test]
fn version_flag() {
    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn invalid_size_fails() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("bad.ndjson");

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("abc")
        .arg("--output")
        .arg(output.to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn zero_size_fails() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("zero.ndjson");

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("0")
        .arg("--output")
        .arg(output.to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn generate_to_stdout() {
    let output = Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("1KB")
        .arg("--stdout")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    for line in stdout.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "invalid JSON on stdout: {line}"
        );
    }
}

#[test]
fn output_and_stdout_conflict() {
    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("1KB")
        .arg("--output")
        .arg("/tmp/test.ndjson")
        .arg("--stdout")
        .assert()
        .failure();
}

#[test]
fn no_output_or_stdout_errors() {
    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("1KB")
        .assert()
        .failure();
}

#[test]
fn seed_flag_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("seeded.ndjson");
    let output_str = output.to_str().unwrap();

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate")
        .arg("1KB")
        .arg("--output")
        .arg(output_str)
        .arg("--seed")
        .arg("42")
        .assert()
        .success();
}
