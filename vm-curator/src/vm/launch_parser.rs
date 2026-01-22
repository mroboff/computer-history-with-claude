use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::qemu_config::*;

/// Parse a launch.sh script and extract QEMU configuration
pub fn parse_launch_script(script_path: &Path, content: &str) -> Result<QemuConfig> {
    let mut config = QemuConfig::default();
    config.raw_script = content.to_string();

    let vm_dir = script_path.parent().unwrap_or(Path::new("."));

    // Extract emulator
    if let Some(emulator) = extract_emulator(content) {
        config.emulator = emulator;
    }

    // Extract memory
    if let Some(mem) = extract_memory(content) {
        config.memory_mb = mem;
    }

    // Extract CPU cores
    if let Some(cores) = extract_cpu_cores(content) {
        config.cpu_cores = cores;
    }

    // Extract CPU model
    config.cpu_model = extract_cpu_model(content);

    // Extract machine type
    config.machine = extract_machine(content);

    // Extract VGA
    if let Some(vga) = extract_vga(content) {
        config.vga = vga;
    }

    // Extract audio devices
    config.audio_devices = extract_audio_devices(content);

    // Check for KVM
    config.enable_kvm = content.contains("-enable-kvm") || content.contains("-accel kvm");

    // Check for UEFI
    config.uefi = content.contains("OVMF") || content.contains("-bios") && content.contains("efi");

    // Check for TPM
    config.tpm = content.contains("-tpmdev") || content.contains("swtpm");

    // Extract disks
    config.disks = extract_disks(content, vm_dir);

    // Extract network config
    config.network = extract_network(content);

    // Extract extra arguments we don't specifically parse
    config.extra_args = extract_extra_args(content);

    Ok(config)
}

/// Extract the QEMU emulator command
fn extract_emulator(content: &str) -> Option<QemuEmulator> {
    let emulators = [
        "qemu-system-x86_64",
        "qemu-system-i386",
        "qemu-system-ppc",
        "qemu-system-m68k",
        "qemu-system-arm",
        "qemu-system-aarch64",
    ];

    for emulator in emulators {
        if content.contains(emulator) {
            return Some(QemuEmulator::from_command(emulator));
        }
    }
    None
}

/// Extract memory configuration
fn extract_memory(content: &str) -> Option<u32> {
    for line in content.lines() {
        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for -m flag
        if let Some(idx) = line.find("-m ") {
            let rest = &line[idx + 3..];
            let value: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(mem) = value.parse::<u32>() {
                // Check for G suffix
                if rest.contains('G') {
                    return Some(mem * 1024);
                }
                // If less than 64, probably gigabytes
                if mem < 64 {
                    return Some(mem * 1024);
                }
                return Some(mem);
            }
        }
    }
    None
}

/// Extract CPU cores
fn extract_cpu_cores(content: &str) -> Option<u32> {
    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for -smp
        if let Some(idx) = line.find("-smp ") {
            let rest = &line[idx + 5..];
            let value: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(cores) = value.parse::<u32>() {
                return Some(cores);
            }
        }
    }
    None
}

/// Extract CPU model
fn extract_cpu_model(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(idx) = line.find("-cpu ") {
            let rest = &line[idx + 5..];
            let model: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '\\')
                .collect();
            if !model.is_empty() {
                return Some(model);
            }
        }
    }
    None
}

/// Extract machine type
fn extract_machine(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(idx) = line.find("-M ") {
            let rest = &line[idx + 3..];
            let machine: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '\\')
                .collect();
            if !machine.is_empty() {
                return Some(machine);
            }
        }

        if let Some(idx) = line.find("-machine ") {
            let rest = &line[idx + 9..];
            let machine: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != ',' && *c != '\\')
                .collect();
            if !machine.is_empty() {
                return Some(machine);
            }
        }
    }
    None
}

/// Extract VGA type
fn extract_vga(content: &str) -> Option<VgaType> {
    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(idx) = line.find("-vga ") {
            let rest = &line[idx + 5..];
            let vga: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '\\')
                .collect();
            if !vga.is_empty() {
                return Some(VgaType::from_str(&vga));
            }
        }
    }
    None
}

/// Extract audio devices
fn extract_audio_devices(content: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    // Check for SoundBlaster 16
    if content.contains("sb16") || content.contains("SB16") {
        devices.push(AudioDevice::Sb16);
    }

    // Check for AC97
    if content.contains("ac97") || content.contains("AC97") {
        devices.push(AudioDevice::Ac97);
    }

    // Check for Intel HDA
    if content.contains("intel-hda") || content.contains("hda-duplex") {
        devices.push(AudioDevice::Hda);
    }

    // Check for ES1370
    if content.contains("es1370") {
        devices.push(AudioDevice::Es1370);
    }

    devices
}

/// Extract disk configurations
fn extract_disks(content: &str, vm_dir: &Path) -> Vec<DiskConfig> {
    let mut disks = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for -hda, -hdb, etc.
        for hd in ["hda", "hdb", "hdc", "hdd"] {
            let pattern = format!("-{} ", hd);
            if let Some(idx) = line.find(&pattern) {
                let rest = &line[idx + pattern.len()..];
                if let Some(path) = extract_path_from_arg(rest) {
                    let full_path = resolve_path(&path, vm_dir);
                    let format = guess_disk_format(&full_path);
                    disks.push(DiskConfig {
                        path: full_path,
                        format,
                        interface: "ide".to_string(),
                    });
                }
            }
        }

        // Look for -drive file=
        if line.contains("-drive") && line.contains("file=") {
            if let Some(path) = extract_drive_file(line) {
                let full_path = resolve_path(&path, vm_dir);
                let format = guess_disk_format(&full_path);
                let interface = if line.contains("if=virtio") {
                    "virtio"
                } else if line.contains("if=scsi") {
                    "scsi"
                } else {
                    "ide"
                };
                disks.push(DiskConfig {
                    path: full_path,
                    format,
                    interface: interface.to_string(),
                });
            }
        }
    }

    disks
}

/// Extract file path from -drive file= argument
fn extract_drive_file(line: &str) -> Option<String> {
    if let Some(idx) = line.find("file=") {
        let rest = &line[idx + 5..];
        // Handle quoted paths
        if rest.starts_with('"') {
            let end = rest[1..].find('"')?;
            return Some(rest[1..=end].to_string());
        }
        // Handle unquoted paths
        let path: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ',' && *c != '\\')
            .collect();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Extract a path from an argument
fn extract_path_from_arg(arg: &str) -> Option<String> {
    let trimmed = arg.trim();
    if trimmed.starts_with('"') {
        let end = trimmed[1..].find('"')?;
        return Some(trimmed[1..=end].to_string());
    }
    if trimmed.starts_with('\'') {
        let end = trimmed[1..].find('\'')?;
        return Some(trimmed[1..=end].to_string());
    }
    let path: String = trimmed
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '\\')
        .collect();
    if !path.is_empty() && !path.starts_with('-') {
        Some(path)
    } else {
        None
    }
}

/// Resolve a path relative to VM directory
fn resolve_path(path: &str, vm_dir: &Path) -> PathBuf {
    let path = path.replace("$DIR", &vm_dir.to_string_lossy());
    let path = path.replace("${DIR}", &vm_dir.to_string_lossy());
    let path = path.replace("$(dirname $0)", &vm_dir.to_string_lossy());

    let p = PathBuf::from(&path);
    if p.is_absolute() {
        p
    } else {
        vm_dir.join(p)
    }
}

/// Guess disk format from file extension
fn guess_disk_format(path: &PathBuf) -> DiskFormat {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(DiskFormat::from_extension)
        .unwrap_or(DiskFormat::Raw)
}

/// Extract network configuration
fn extract_network(content: &str) -> Option<NetworkConfig> {
    let mut config = NetworkConfig::default();
    let mut has_network = false;

    for line in content.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for network model
        if line.contains("-net nic") || line.contains("-netdev") || line.contains("-nic") {
            has_network = true;

            if line.contains("model=virtio") {
                config.model = "virtio-net".to_string();
            } else if line.contains("model=e1000") {
                config.model = "e1000".to_string();
            } else if line.contains("model=rtl8139") {
                config.model = "rtl8139".to_string();
            }
        }

        // Check for user networking
        if line.contains("-net user") || line.contains("user,") {
            config.user_net = true;
        }

        // Check for bridge
        if line.contains("-net bridge") || line.contains("bridge,") {
            config.user_net = false;
            // Extract bridge name if present
            if let Some(idx) = line.find("br=") {
                let rest = &line[idx + 3..];
                let bridge: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                config.bridge = Some(bridge);
            }
        }
    }

    if has_network || content.contains("-net") || content.contains("-nic") {
        Some(config)
    } else {
        None
    }
}

/// Extract extra arguments we don't specifically handle
fn extract_extra_args(content: &str) -> Vec<String> {
    let mut args = Vec::new();

    // Look for display settings
    if content.contains("-display gtk") {
        args.push("-display gtk".to_string());
    } else if content.contains("-display sdl") {
        args.push("-display sdl".to_string());
    } else if content.contains("-display vnc") {
        args.push("-display vnc".to_string());
    }

    // Look for USB
    if content.contains("-usb") {
        args.push("-usb".to_string());
    }

    // Look for RTC settings
    if content.contains("-rtc base=localtime") {
        args.push("-rtc base=localtime".to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_memory() {
        assert_eq!(extract_memory("-m 512"), Some(512));
        assert_eq!(extract_memory("-m 2G"), Some(2048));
        assert_eq!(extract_memory("qemu -m 1024 -cpu host"), Some(1024));
    }

    #[test]
    fn test_extract_emulator() {
        assert_eq!(
            extract_emulator("#!/bin/bash\nqemu-system-i386 -m 512"),
            Some(QemuEmulator::I386)
        );
        assert_eq!(
            extract_emulator("qemu-system-ppc -M mac99"),
            Some(QemuEmulator::Ppc)
        );
    }

    #[test]
    fn test_extract_vga() {
        assert_eq!(
            extract_vga("-vga cirrus -m 512"),
            Some(VgaType::Cirrus)
        );
        assert_eq!(
            extract_vga("-vga virtio"),
            Some(VgaType::Virtio)
        );
    }
}
