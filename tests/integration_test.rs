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
fn generate_openapi_creates_valid_ndjson() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("spec.yaml");
    let output = dir.path().join("openapi.ndjson");

    std::fs::write(
        &spec,
        r#"openapi: 3.0.3
info:
    title: NDJSON Test
    version: 1.0.0
paths: {}
components:
    schemas:
        Event:
            type: object
            properties:
                id:
                    type: integer
                    minimum: 1
                    maximum: 100000
                status:
                    type: string
                    enum: [new, processing, complete]
                active:
                    type: boolean
                tags:
                    type: array
                    items:
                        type: string
                nested:
                    type: object
                    properties:
                        score:
                            type: number
                            minimum: 0
                            maximum: 1
"#,
    )
    .unwrap();

    Command::cargo_bin("ndjson-gen")
        .unwrap()
        .arg("generate-openapi")
        .arg("2KB")
        .arg("--spec")
        .arg(spec.to_str().unwrap())
        .arg("--schema")
        .arg("Event")
        .arg("--output")
        .arg(output.to_str().unwrap())
        .assert()
        .success();

    let contents = fs::read_to_string(&output).unwrap();
    assert!(!contents.is_empty());

    for line in contents.lines() {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("status").is_some());
        assert!(parsed.get("active").is_some());
        assert!(parsed.get("tags").is_some());
        assert!(parsed.get("nested").is_some());
    }
}
