use std::process::Command;

#[test]
fn refuses_to_start_without_database_url() {
    let output = Command::new(env!("CARGO_BIN_EXE_api-example"))
        .env_remove("DATABASE_URL")
        .output()
        .expect("run server without database configuration");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("DATABASE_URL must be set"));
}
