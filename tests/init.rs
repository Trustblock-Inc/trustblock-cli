mod common;

use assert_cmd::Command;

use serial_test::serial;

use common::constants::{ CLI_PATH, FIXTURES_DIR };

use predicates::prelude::*;

#[test]
#[ignore]
#[serial("Serial because it mutates shared state")]
fn test_init_no_args_success() -> eyre::Result<()> {
    // Cleans .trustblock folder
    Command::cargo_bin("trustblock")?.arg("clean").assert().success();

    let home_dir = dirs::home_dir().expect("Could not find home directory");

    let trustblock_dir = home_dir.join(CLI_PATH);

    let env_path = trustblock_dir.join(".env");

    let init_fixture_path = format!("{}{}", FIXTURES_DIR, "init.stdout");
    let env_fixture_path = format!("{}{}", FIXTURES_DIR, "env_file_no_values.stdout");

    //Write to fixture file
    std::fs::write(
        &init_fixture_path,
        format!(
            "Generating .trustblock folder...\n\nCreated .env file at \"{}/.env\"\n",
            trustblock_dir.to_string_lossy()
        )
    )?;

    Command::cargo_bin("trustblock")?
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::path::eq_file(init_fixture_path));

    // Checks if .trustblock folder exists
    predicate::path::exists().eval(&trustblock_dir);

    // Checks if .env file is equal to fixture
    predicate::path::eq_file(env_fixture_path).eval(env_path.as_path());

    let predicate_regenerate_file = predicate::str::contains(
        format!(".env file already exists at {env_path:?}")
    );

    // Tries to generate a folder again
    Command::cargo_bin("trustblock")?
        .arg("init")
        .assert()
        .success()
        .stdout(predicate_regenerate_file);

    Ok(())
}

#[test]
#[ignore]
#[serial("Serial because it mutates shared state")]
fn test_init_args_success() -> eyre::Result<()> {
    // Cleans .trustblock folder (if it exists)
    Command::cargo_bin("trustblock")?.arg("clean").assert().success();

    let home_dir = dirs::home_dir().expect("Could not find home directory");

    let trustblock_dir = home_dir.join(CLI_PATH);

    let env_path = trustblock_dir.join(".env");

    let init_fixture_path = format!("{}{}", FIXTURES_DIR, "init.stdout");
    let env_fixture_path = format!("{}{}", FIXTURES_DIR, "env_file_with_values.stdout");

    //Write to fixture file
    std::fs::write(
        &init_fixture_path,
        format!(
            "Generating .trustblock folder...\n\nCreated .env file at \"{}/.env\"\n",
            trustblock_dir.to_string_lossy()
        )
    )?;

    Command::cargo_bin("trustblock")?
        .arg("init")
        .arg("-a")
        .arg("some_api_key")
        .assert()
        .success()
        .stdout(predicate::path::eq_file(init_fixture_path));

    // Checks if .trustblock folder exists
    predicate::path::exists().eval(&trustblock_dir);

    // Checks if .env file is equal to fixture
    predicate::path::eq_file(env_fixture_path).eval(env_path.as_path());

    Ok(())
}

#[test]
fn test_init_args_incorrect_args_fail() -> eyre::Result<()> {
    Command::cargo_bin("trustblock")?
        .arg("init")
        .arg("-d")
        .arg("some_private_key")
        .arg("-a")
        .arg("some_api_key")
        .assert()
        .failure();

    Command::cargo_bin("trustblock")?
        .arg("init")
        .arg("-p")
        .arg("some_private_key")
        .arg("-c")
        .arg("some_api_key")
        .assert()
        .failure();

    Command::cargo_bin("trustblock")?
        .arg("init")
        .arg("--privadte-key")
        .arg("some_private_key")
        .arg("-a")
        .arg("some_api_key")
        .assert()
        .failure();

    Command::cargo_bin("trustblock")?
        .arg("init")
        .arg("--private-key")
        .arg("some_private_key")
        .arg("--auth-tokens")
        .arg("some_api_key")
        .assert()
        .failure();

    Ok(())
}