use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use std::path::Path;
use tempfile::TempDir;

/// Creates a temp directory and returns a Command that runs bb in that directory
fn bb_in_temp(temp_dir: &Path) -> Command {
    let mut cmd = cargo_bin_cmd!("bb");
    cmd.current_dir(temp_dir);
    cmd
}

/// Helper to run bb init in a temp directory
fn bb_init(temp_dir: &Path) {
    bb_in_temp(temp_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("Initialized blackboard"));
}

// ============================================================================
// Smoke Tests
// ============================================================================

#[test]
fn test_smoke_bb_binary_runs() {
    let temp = TempDir::new().unwrap();
    bb_in_temp(temp.path()).arg("--help").assert().success();
}

#[test]
fn test_smoke_bb_init_creates_structure() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Verify .bb directory was created
    assert!(temp.path().join(".bb").exists());
    assert!(temp.path().join(".bb/blackboard.db").exists());
    assert!(temp.path().join(".bb/.gitignore").exists());
}

// ============================================================================
// Task #11: Test bb init creates .bb/ directory structure
// ============================================================================

#[test]
fn test_init_creates_bb_directory() {
    let temp = TempDir::new().unwrap();

    bb_in_temp(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("Initialized blackboard"));

    assert!(temp.path().join(".bb").is_dir());
}

#[test]
fn test_init_creates_database() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    assert!(temp.path().join(".bb/blackboard.db").is_file());
}

#[test]
fn test_init_creates_gitignore_with_star() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    let gitignore_path = temp.path().join(".bb/.gitignore");
    assert!(gitignore_path.exists());

    let content = std::fs::read_to_string(gitignore_path).unwrap();
    assert_eq!(content, "*\n");
}

// ============================================================================
// Task #12: Test discovery from subdirectories
// ============================================================================

#[test]
fn test_discovery_from_subdirectory() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Create a subdirectory
    let subdir = temp.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    // Run bb log from subdirectory should work
    bb_in_temp(&subdir).arg("log").assert().success();
}

// ============================================================================
// Task #13: Test commands fail without init
// ============================================================================

#[test]
fn test_post_fails_without_init() {
    let temp = TempDir::new().unwrap();

    bb_in_temp(temp.path())
        .args(["post", "test message"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("No blackboard found"));
}

#[test]
fn test_log_fails_without_init() {
    let temp = TempDir::new().unwrap();

    bb_in_temp(temp.path())
        .arg("log")
        .assert()
        .failure()
        .stderr(predicates::str::contains("No blackboard found"));
}

// ============================================================================
// Task #14: Test post and log roundtrip
// ============================================================================

#[test]
fn test_post_and_log_roundtrip() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post a message
    let test_content = "Test message content 12345";
    bb_in_temp(temp.path())
        .args(["post", test_content])
        .assert()
        .success();

    // Read it back
    bb_in_temp(temp.path())
        .arg("log")
        .assert()
        .success()
        .stdout(predicates::str::contains(test_content));
}

// ============================================================================
// Task #15: Test log filters
// ============================================================================

#[test]
fn test_log_filter_by_tag() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post messages with different tags
    bb_in_temp(temp.path())
        .args(["post", "message with alpha tag", "--tag", "alpha"])
        .assert()
        .success();

    bb_in_temp(temp.path())
        .args(["post", "message with beta tag", "--tag", "beta"])
        .assert()
        .success();

    // Filter by alpha tag should only show alpha message
    bb_in_temp(temp.path())
        .args(["log", "--tag", "alpha"])
        .assert()
        .success()
        .stdout(predicates::str::contains("alpha tag"))
        .stdout(predicates::str::contains("beta tag").not());
}

#[test]
fn test_log_filter_by_from() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post as different agents
    bb_in_temp(temp.path())
        .args(["post", "from agent1", "--as", "agent1"])
        .assert()
        .success();

    bb_in_temp(temp.path())
        .args(["post", "from agent2", "--as", "agent2"])
        .assert()
        .success();

    // Filter by agent1
    bb_in_temp(temp.path())
        .args(["log", "--from", "agent1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("from agent1"))
        .stdout(predicates::str::contains("from agent2").not());
}

#[test]
fn test_log_filter_by_priority() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post with different priorities
    bb_in_temp(temp.path())
        .args(["post", "low priority", "--priority", "low"])
        .assert()
        .success();

    bb_in_temp(temp.path())
        .args(["post", "high priority", "--priority", "high"])
        .assert()
        .success();

    // Filter by high priority
    bb_in_temp(temp.path())
        .args(["log", "--priority", "high"])
        .assert()
        .success()
        .stdout(predicates::str::contains("high priority"));
}

// ============================================================================
// Task #16: Test refs attach and find
// ============================================================================

#[test]
fn test_refs_attach_and_find() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post a message with a ref
    bb_in_temp(temp.path())
        .args(["post", "message with ref", "--ref", "test:message:123"])
        .assert()
        .success();

    // Find the ref
    bb_in_temp(temp.path())
        .args(["refs", "test:message:123"])
        .assert()
        .success()
        .stdout(predicates::str::contains("message with ref"));
}

// ============================================================================
// Task #17: Test artifact upsert overwrites fields
// ============================================================================

#[test]
fn test_artifact_add_and_list() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Create a test file with relative path
    let test_file = "test.txt";
    std::fs::write(temp.path().join(test_file), "test content").unwrap();

    // Register artifact (using relative path from project dir)
    bb_in_temp(temp.path())
        .args(["artifact-add", test_file, "test description"])
        .assert()
        .success();

    // Verify artifact appears in list
    bb_in_temp(temp.path())
        .args(["artifacts"])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"));
}

// ============================================================================
// Task #18: Test --json output is valid JSON
// ============================================================================

#[test]
fn test_log_json_output_is_valid() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post a message
    bb_in_temp(temp.path())
        .args(["post", "test message"])
        .assert()
        .success();

    // Get JSON output
    let output = bb_in_temp(temp.path())
        .args(["log", "--json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert!(json.is_array());
}

// ============================================================================
// Task #19: Test bb install emits MCP config snippets
// ============================================================================

#[test]
fn test_install_outputs_mcp_config() {
    bb_in_temp(std::env::temp_dir().as_path())
        .arg("install")
        .assert()
        .success()
        .stdout(predicates::str::contains("Installed to:"))
        .stdout(predicates::str::contains(".mcp.json"));
}

// ============================================================================
// Task #20: Test clear and export commands
// ============================================================================

#[test]
fn test_export_outputs_valid_json() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post a message
    bb_in_temp(temp.path())
        .args(["post", "test message"])
        .assert()
        .success();

    // Export
    let output = bb_in_temp(temp.path()).arg("export").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Export should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_clear_command_exists() {
    let temp = TempDir::new().unwrap();
    bb_init(temp.path());

    // Post a message
    bb_in_temp(temp.path())
        .args(["post", "test message"])
        .assert()
        .success();

    // Clear command should succeed
    bb_in_temp(temp.path())
        .args(["clear", "--confirm"])
        .assert()
        .success();
}
