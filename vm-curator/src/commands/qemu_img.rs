use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Create a new qcow2 disk image
pub fn create_disk(path: &Path, size: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "create",
            "-f", "qcow2",
            path.to_str().unwrap_or(""),
            size,
        ])
        .output()
        .context("Failed to run qemu-img create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create disk: {}", stderr);
    }

    Ok(())
}

/// Create a disk with a backing file
pub fn create_disk_with_backing(path: &Path, backing: &Path, backing_format: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "create",
            "-f", "qcow2",
            "-F", backing_format,
            "-b", backing.to_str().unwrap_or(""),
            path.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to run qemu-img create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create disk with backing file: {}", stderr);
    }

    Ok(())
}

/// Convert a disk image to a different format
pub fn convert_disk(source: &Path, dest: &Path, format: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "convert",
            "-f", "auto",
            "-O", format,
            source.to_str().unwrap_or(""),
            dest.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to run qemu-img convert")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to convert disk: {}", stderr);
    }

    Ok(())
}

/// Resize a disk image
pub fn resize_disk(path: &Path, size: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "resize",
            path.to_str().unwrap_or(""),
            size,
        ])
        .output()
        .context("Failed to run qemu-img resize")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to resize disk: {}", stderr);
    }

    Ok(())
}

/// Check disk integrity
pub fn check_disk(path: &Path) -> Result<DiskCheckResult> {
    let output = Command::new("qemu-img")
        .args([
            "check",
            path.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to run qemu-img check")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(DiskCheckResult {
        success: output.status.success(),
        output: format!("{}\n{}", stdout, stderr).trim().to_string(),
        errors: !output.status.success(),
    })
}

/// Disk check result
#[derive(Debug)]
pub struct DiskCheckResult {
    pub success: bool,
    pub output: String,
    pub errors: bool,
}

/// Compact a qcow2 disk (remove unused space)
pub fn compact_disk(path: &Path) -> Result<()> {
    // First, convert to a temporary file
    let temp_path = path.with_extension("qcow2.tmp");

    let output = Command::new("qemu-img")
        .args([
            "convert",
            "-O", "qcow2",
            path.to_str().unwrap_or(""),
            temp_path.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to compact disk")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to compact disk: {}", stderr);
    }

    // Replace original with compacted version
    std::fs::rename(&temp_path, path)
        .context("Failed to replace original disk with compacted version")?;

    Ok(())
}

/// Rebase a disk to a new backing file
pub fn rebase_disk(path: &Path, new_backing: &Path, backing_format: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "rebase",
            "-b", new_backing.to_str().unwrap_or(""),
            "-F", backing_format,
            path.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to run qemu-img rebase")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to rebase disk: {}", stderr);
    }

    Ok(())
}

/// Commit changes from overlay to backing file
pub fn commit_disk(path: &Path) -> Result<()> {
    let output = Command::new("qemu-img")
        .args([
            "commit",
            path.to_str().unwrap_or(""),
        ])
        .output()
        .context("Failed to run qemu-img commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to commit disk: {}", stderr);
    }

    Ok(())
}
