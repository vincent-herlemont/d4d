use predicates::prelude::Predicate;
use predicates::str::contains;
use test_utils::init;
use test_utils::{
    HOME_CFG_FILE, PRIVATE_ENV_DEV_FILE, PRIVATE_ENV_DIR, PROJECT_CFG_FILE, PROJECT_ENV_DIR,
    PROJECT_ENV_EXAMPLE_1_FILE, PROJECT_ENV_EXAMPLE_2_FILE,
};

mod test_utils;

#[test]
fn cmd_var() {
    let mut e = init("cmd_var");
    e.add_file(
        PROJECT_CFG_FILE,
        r#"
setups:
  - name: setup_1
    file: run.sh
    array_vars:
        ALL: "VAR.*"
    vars: [ VAR_A, CONFIG ]
    "#,
    );
    e.add_file(
        PROJECT_ENV_EXAMPLE_1_FILE,
        r#"VAR_A=VALUE1
VAR_B=VALUE1
CONFIG=AZE
"#,
    );
    e.add_file(
        PROJECT_ENV_EXAMPLE_2_FILE,
        r#"VAR_A=VALUE2
VAR_B=VALUE2
CONFIG=AZE
"#,
    );
    e.setup();
    e.set_update_file_time(PROJECT_ENV_EXAMPLE_1_FILE).unwrap();

    let mut command = e.command(env!("CARGO_PKG_NAME"));
    let r = command
        .env("RUST_LOG", "debug")
        .arg("var")
        .args(&["-s", "setup_1"])
        .assert()
        .to_string();

    assert!(contains(
        r#"┌─────────────────┬──────────┬──────────┐
│                 │ example1 │ example2 │
│ all    │ ALL (VAR.*)       │          │
│        │ VAR_A  │ VALUE1   │ VALUE2   │
│        │ VAR_B  │ VALUE1   │ VALUE2   │
│ var_a  │ VAR_A  │ VALUE1   │ VALUE2   │
│ config │ CONFIG │ AZE      │ AZE      │
└────────┴────────┴──────────┴──────────┘
"#,
    )
    .eval(&r));
}