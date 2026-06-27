use assert_cmd::Command;

#[test]
fn example_default() {
    Command::cargo_bin("rust-cli-template")
        .unwrap()
        .arg("example")
        .assert()
        .stdout("Hello, world!\n");
}

#[test]
fn example_custom_name() {
    Command::cargo_bin("rust-cli-template")
        .unwrap()
        .arg("example")
        .arg("Rust")
        .assert()
        .stdout("Hello, Rust!\n");
}

#[test]
fn version_flag() {
    Command::cargo_bin("rust-cli-template")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}
