// tests/integration.rs
use std::fs::{self, File};
use std::process::Command;
use tempfile::TempDir;

fn create_test_structure(dir: &std::path::Path) {
    // Create a PSX-style game directory
    let psx = dir.join("psx").join("Final Fantasy VII");
    fs::create_dir_all(&psx).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 1).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 1).bin")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 2).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 2).bin")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 3).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 3).bin")).unwrap();

    // Create an Amiga-style floppy directory
    let amiga = dir.join("amiga").join("Monkey Island");
    fs::create_dir_all(&amiga).unwrap();
    File::create(amiga.join("Monkey Island (Disk 1).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 2).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 3).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 4).adf")).unwrap();
}

#[test]
fn test_basic_m3u_creation() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success(), "Command failed: {:?}", output);

    // Check that m3u was created
    let m3u_path = dir.path().join("psx/Final Fantasy VII/Final Fantasy VII.m3u");
    assert!(m3u_path.exists(), "M3U file not created");

    let content = fs::read_to_string(&m3u_path).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 disc entries");
    assert!(lines[0].contains("Disc 1"));
    assert!(lines[1].contains("Disc 2"));
    assert!(lines[2].contains("Disc 3"));
}

#[test]
fn test_floppy_grouping() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg(dir.path().join("amiga"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    // Floppy mode: all disks in one m3u
    let m3u_path = dir.path().join("amiga/Monkey Island/Monkey Island.m3u");
    assert!(m3u_path.exists());

    let content = fs::read_to_string(&m3u_path).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 4, "Should have 4 floppy entries");
}

#[test]
fn test_relative_paths() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--relative")
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    let m3u_path = dir.path().join("psx/Final Fantasy VII/Final Fantasy VII.m3u");
    let content = fs::read_to_string(&m3u_path).unwrap();

    // Relative paths should not start with /
    for line in content.lines() {
        assert!(!line.starts_with('/'), "Path should be relative: {}", line);
    }
}

#[test]
fn test_children_mode() {
    let dir = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--children")
        .arg(dir.path())
        .arg(dest.path())
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    // Should have created psx/ and amiga/ dirs in destination
    assert!(dest.path().join("psx").exists());
    assert!(dest.path().join("amiga").exists());

    // M3Us should be in those directories
    assert!(dest.path().join("psx/Final Fantasy VII.m3u").exists());
    assert!(dest.path().join("amiga/Monkey Island.m3u").exists());
}

#[test]
fn test_quiet_mode() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--quiet")
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());
    // Quiet mode should have no stdout (progress goes to stderr)
    assert!(output.stdout.is_empty());
}
