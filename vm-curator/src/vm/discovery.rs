use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::launch_parser::parse_launch_script;
use super::qemu_config::QemuConfig;

/// A discovered VM in the library
#[derive(Debug, Clone)]
pub struct DiscoveredVm {
    /// Directory name (e.g., "windows-95")
    pub id: String,
    /// Full path to VM directory
    pub path: PathBuf,
    /// Path to launch.sh
    pub launch_script: PathBuf,
    /// Parsed QEMU configuration
    pub config: QemuConfig,
    /// Whether the VM has been parsed successfully
    pub parse_success: bool,
    /// Parse error message if failed
    pub parse_error: Option<String>,
}

impl DiscoveredVm {
    /// Get a display name from the directory name
    pub fn display_name(&self) -> String {
        self.id
            .replace('-', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars: Vec<char> = word.chars().collect();
                if let Some(first) = chars.first_mut() {
                    *first = first.to_ascii_uppercase();
                }
                chars.into_iter().collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Scan the VM library directory for VMs
pub fn discover_vms(library_path: &Path) -> Result<Vec<DiscoveredVm>> {
    let mut vms = Vec::new();

    if !library_path.exists() {
        return Ok(vms);
    }

    let entries = std::fs::read_dir(library_path)
        .with_context(|| format!("Failed to read VM library at {:?}", library_path))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let launch_script = path.join("launch.sh");
        if !launch_script.exists() {
            continue;
        }

        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to parse the launch script
        let script_content = std::fs::read_to_string(&launch_script)
            .unwrap_or_default();

        let (config, parse_success, parse_error) = match parse_launch_script(&launch_script, &script_content) {
            Ok(cfg) => (cfg, true, None),
            Err(e) => {
                let mut default_config = QemuConfig::default();
                default_config.raw_script = script_content;
                (default_config, false, Some(e.to_string()))
            }
        };

        vms.push(DiscoveredVm {
            id,
            path,
            launch_script,
            config,
            parse_success,
            parse_error,
        });
    }

    // Sort by display name
    vms.sort_by(|a, b| a.display_name().cmp(&b.display_name()));

    Ok(vms)
}

/// Group VMs by category (extracted from naming conventions)
pub fn group_vms_by_category(vms: &[DiscoveredVm]) -> Vec<(&'static str, Vec<&DiscoveredVm>)> {
    let mut windows: Vec<&DiscoveredVm> = Vec::new();
    let mut mac: Vec<&DiscoveredVm> = Vec::new();
    let mut linux: Vec<&DiscoveredVm> = Vec::new();
    let mut other: Vec<&DiscoveredVm> = Vec::new();

    for vm in vms {
        let id_lower = vm.id.to_lowercase();
        if id_lower.starts_with("windows") || id_lower.contains("dos") || id_lower.starts_with("my-first") {
            windows.push(vm);
        } else if id_lower.starts_with("mac") {
            mac.push(vm);
        } else if id_lower.starts_with("linux")
            || id_lower.contains("fedora")
            || id_lower.contains("ubuntu")
            || id_lower.contains("debian")
            || id_lower.contains("arch")
        {
            linux.push(vm);
        } else {
            other.push(vm);
        }
    }

    let mut groups = Vec::new();
    if !windows.is_empty() {
        groups.push(("Windows / DOS", windows));
    }
    if !mac.is_empty() {
        groups.push(("Macintosh", mac));
    }
    if !linux.is_empty() {
        groups.push(("Linux", linux));
    }
    if !other.is_empty() {
        groups.push(("Other", other));
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        let vm = DiscoveredVm {
            id: "windows-95".to_string(),
            path: PathBuf::from("/test"),
            launch_script: PathBuf::from("/test/launch.sh"),
            config: QemuConfig::default(),
            parse_success: true,
            parse_error: None,
        };
        assert_eq!(vm.display_name(), "Windows 95");
    }
}
