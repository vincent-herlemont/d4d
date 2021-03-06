use predicates::prelude::Predicate;
use predicates::str::contains;

use short::BIN_NAME;
use test_utils::init;
use test_utils::{PROJECT_CFG_FILE, PROJECT_ENV_EXAMPLE_1_FILE, PROJECT_ENV_EXAMPLE_2_FILE};

mod test_utils;

const MOCK_EDITOR_FILE: &'static str = "mock_editor.sh";

#[test]
fn cmd_edit() {
    let mut e = init("cmd_env_edit");

    e.add_file(PROJECT_ENV_EXAMPLE_1_FILE, r#"VAR1=VALUE1"#);
    e.add_file(
        PROJECT_CFG_FILE,
        r#"
setups:
  setup_1:
    file: run.sh
        "#,
    );
    e.add_file(
        MOCK_EDITOR_FILE,
        r#"#!/bin/bash
echo -e "\nVAR2=VALUE2" >> $1
        "#,
    );
    e.setup();
    e.set_exec_permission(MOCK_EDITOR_FILE).unwrap();

    let mock_editor_file_abs = e.path().unwrap().join(MOCK_EDITOR_FILE);

    let mut command = e.command(BIN_NAME).unwrap();
    let r = command
        .env("RUST_LOG", "debug")
        .arg("edit")
        .arg("example1")
        .args(vec!["-s", "setup_1"])
        .args(vec![
            "--editor",
            mock_editor_file_abs.to_string_lossy().into_owned().as_str(),
        ])
        .assert()
        .to_string();

    assert!(contains("`example1` edited").count(1).eval(&r));

    let r = e.read_file(PROJECT_ENV_EXAMPLE_1_FILE);
    assert!(contains("VAR2=VALUE2").count(1).eval(&r));
}

#[test]
fn cmd_edit_with_sync() {
    let mut e = init("cmd_env_edit");
    e.add_file(PROJECT_ENV_EXAMPLE_1_FILE, r#"VAR1=VALUE1"#);
    e.add_file(PROJECT_ENV_EXAMPLE_2_FILE, r#"VAR1=VALUE1"#);
    e.add_file(
        PROJECT_CFG_FILE,
        r#"
setups:
  setup_1:
    file: run.sh
        "#,
    );

    e.add_file(
        MOCK_EDITOR_FILE,
        r#"#!/bin/bash
echo -e "\nVAR2=VALUE2" >> $1
        "#,
    );
    e.setup();
    e.set_exec_permission(MOCK_EDITOR_FILE).unwrap();

    let mock_editor_file_abs = e.path().unwrap().join(MOCK_EDITOR_FILE);

    let mut command = e.command(BIN_NAME).unwrap();
    let r = command
        .env("RUST_LOG", "debug")
        .arg("edit")
        .arg("example1")
        .arg("--copy")
        .args(vec!["-s", "setup_1"])
        .args(vec![
            "--editor",
            mock_editor_file_abs.to_string_lossy().into_owned().as_str(),
        ])
        .assert()
        .to_string();

    assert!(contains("`example1` edited").count(1).eval(&r));

    let r = e.read_file(PROJECT_ENV_EXAMPLE_1_FILE);
    assert!(contains("VAR2=VALUE2").count(1).eval(&r));
    let r = e.read_file(PROJECT_ENV_EXAMPLE_2_FILE);
    assert!(contains("VAR2=VALUE2").count(1).eval(&r));
}
