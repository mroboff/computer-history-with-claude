use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// A snapshot of a VM disk
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub size: String,
    pub date: String,
    pub vm_clock: String,
}

/// List snapshots for a qcow2 disk image
pub fn list_snapshots(disk_path: &Path) -> Result<Vec<Snapshot>> {
    let output = Command::new("qemu-img")
        .args(["snapshot", "-l", disk_path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run qemu-img snapshot -l")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no snapshots") || output.stdout.is_empty() {
            return Ok(Vec::new());
        }
        bail!("qemu-img snapshot -l failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_snapshot_list(&stdout)
}

/// Parse the output of qemu-img snapshot -l
fn parse_snapshot_list(output: &str) -> Result<Vec<Snapshot>> {
    let mut snapshots = Vec::new();
    let mut in_table = false;

    for line in output.lines() {
        let line = line.trim();

        // Skip header lines
        if line.starts_with("Snapshot") || line.starts_with("ID") || line.starts_with("--") {
            in_table = true;
            continue;
        }

        if !in_table || line.is_empty() {
            continue;
        }

        // Parse snapshot line: ID TAG VM SIZE DATE VM CLOCK
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            // Try to reconstruct the fields - the date might have spaces
            let id = parts[0].to_string();
            let name = parts[1].to_string();
            let size = parts[2].to_string();

            // Date is typically in format like "2024-01-15 10:30:45"
            let date = if parts.len() >= 5 {
                format!("{} {}", parts[3], parts.get(4).unwrap_or(&""))
            } else {
                parts.get(3).unwrap_or(&"").to_string()
            };

            let vm_clock = parts.last().unwrap_or(&"").to_string();

            snapshots.push(Snapshot {
                id,
                name,
                size,
                date,
                vm_clock,
            });
        }
    }

    Ok(snapshots)
}

/// Create a new snapshot
pub fn create_snapshot(disk_path: &Path, name: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args(["snapshot", "-c", name, disk_path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run qemu-img snapshot -c")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create snapshot: {}", stderr);
    }

    Ok(())
}

/// Restore (apply) a snapshot
pub fn restore_snapshot(disk_path: &Path, name: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args(["snapshot", "-a", name, disk_path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run qemu-img snapshot -a")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to restore snapshot: {}", stderr);
    }

    Ok(())
}

/// Delete a snapshot
pub fn delete_snapshot(disk_path: &Path, name: &str) -> Result<()> {
    let output = Command::new("qemu-img")
        .args(["snapshot", "-d", name, disk_path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run qemu-img snapshot -d")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to delete snapshot: {}", stderr);
    }

    Ok(())
}

/// Get information about a disk image
pub fn get_disk_info(disk_path: &Path) -> Result<DiskInfo> {
    let output = Command::new("qemu-img")
        .args(["info", disk_path.to_str().unwrap_or("")])
        .output()
        .context("Failed to run qemu-img info")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get disk info: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_disk_info(&stdout)
}

/// Disk image information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub format: String,
    pub virtual_size: String,
    pub disk_size: String,
    pub cluster_size: Option<String>,
    pub backing_file: Option<String>,
}

/// Parse qemu-img info output
fn parse_disk_info(output: &str) -> Result<DiskInfo> {
    let mut format = String::new();
    let mut virtual_size = String::new();
    let mut disk_size = String::new();
    let mut cluster_size = None;
    let mut backing_file = None;

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("file format:") {
            format = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("virtual size:") {
            virtual_size = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("disk size:") {
            disk_size = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("cluster_size:") {
            cluster_size = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("backing file:") {
            backing_file = Some(value.trim().to_string());
        }
    }

    Ok(DiskInfo {
        format,
        virtual_size,
        disk_size,
        cluster_size,
        backing_file,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snapshot_list() {
        let output = r#"
Snapshot list:
ID        TAG               VM SIZE                DATE       VM CLOCK
1         fresh-install        512M 2024-01-15 10:30:45   00:05:30.123
2         after-drivers        768M 2024-01-16 14:20:00   00:15:45.456
"#;
        let snapshots = parse_snapshot_list(output).unwrap();
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].name, "fresh-install");
        assert_eq!(snapshots[1].name, "after-drivers");
    }
}
