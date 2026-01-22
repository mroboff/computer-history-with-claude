use anyhow::{Context, Result};

/// Represents a USB device
#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: String,
    pub product_name: String,
    pub bus_num: u8,
    pub dev_num: u8,
    pub device_class: u8,
}

impl UsbDevice {
    /// Check if this is a hub device
    pub fn is_hub(&self) -> bool {
        // USB Hub class is 0x09
        self.device_class == 0x09
    }

    /// Get a display string for this device
    pub fn display_name(&self) -> String {
        if !self.product_name.is_empty() {
            if !self.vendor_name.is_empty() {
                format!("{} {}", self.vendor_name, self.product_name)
            } else {
                self.product_name.clone()
            }
        } else {
            format!("USB Device {:04x}:{:04x}", self.vendor_id, self.product_id)
        }
    }

    /// Generate QEMU passthrough arguments
    pub fn to_qemu_args(&self) -> Vec<String> {
        vec![
            "-device".to_string(),
            format!(
                "usb-host,vendorid=0x{:04x},productid=0x{:04x}",
                self.vendor_id, self.product_id
            ),
        ]
    }
}

/// Enumerate USB devices using libudev
pub fn enumerate_usb_devices() -> Result<Vec<UsbDevice>> {
    // Try using libudev, fall back to sysfs
    let mut devices = enumerate_via_udev()
        .unwrap_or_else(|_| enumerate_via_sysfs().unwrap_or_default());

    // Filter out hubs and root hubs
    devices.retain(|d| !d.is_hub());

    Ok(devices)
}

/// Enumerate using libudev
fn enumerate_via_udev() -> Result<Vec<UsbDevice>> {
    use libudev::Context;

    let context = Context::new().context("Failed to create udev context")?;
    let mut enumerator = libudev::Enumerator::new(&context)
        .context("Failed to create udev enumerator")?;

    enumerator.match_subsystem("usb")
        .context("Failed to match USB subsystem")?;

    let mut devices = Vec::new();

    for device in enumerator.scan_devices()? {
        // Only process USB devices (not interfaces)
        if device.devtype().map(|t| t == "usb_device").unwrap_or(false) {
            let vendor_id = device
                .attribute_value("idVendor")
                .and_then(|v| v.to_str())
                .and_then(|s| u16::from_str_radix(s, 16).ok())
                .unwrap_or(0);

            let product_id = device
                .attribute_value("idProduct")
                .and_then(|v| v.to_str())
                .and_then(|s| u16::from_str_radix(s, 16).ok())
                .unwrap_or(0);

            // Skip root hubs (usually vendor 0x1d6b)
            if vendor_id == 0x1d6b {
                continue;
            }

            let vendor_name = device
                .attribute_value("manufacturer")
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_string();

            let product_name = device
                .attribute_value("product")
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_string();

            let bus_num = device
                .attribute_value("busnum")
                .and_then(|v| v.to_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let dev_num = device
                .attribute_value("devnum")
                .and_then(|v| v.to_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let device_class = device
                .attribute_value("bDeviceClass")
                .and_then(|v| v.to_str())
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .unwrap_or(0);

            devices.push(UsbDevice {
                vendor_id,
                product_id,
                vendor_name,
                product_name,
                bus_num,
                dev_num,
                device_class,
            });
        }
    }

    Ok(devices)
}

/// Fallback enumeration via /sys/bus/usb/devices
fn enumerate_via_sysfs() -> Result<Vec<UsbDevice>> {
    let mut devices = Vec::new();
    let sysfs_path = std::path::Path::new("/sys/bus/usb/devices");

    if !sysfs_path.exists() {
        return Ok(devices);
    }

    for entry in std::fs::read_dir(sysfs_path)? {
        let entry = entry?;
        let path = entry.path();

        // Skip entries that look like interfaces (contain ':')
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.contains(':') {
            continue;
        }

        // Try to read device attributes
        let vendor_id = read_sysfs_hex(&path, "idVendor").unwrap_or(0);
        let product_id = read_sysfs_hex(&path, "idProduct").unwrap_or(0);

        // Skip if no valid IDs
        if vendor_id == 0 && product_id == 0 {
            continue;
        }

        // Skip root hubs
        if vendor_id == 0x1d6b {
            continue;
        }

        let vendor_name = read_sysfs_string(&path, "manufacturer").unwrap_or_default();
        let product_name = read_sysfs_string(&path, "product").unwrap_or_default();
        let bus_num = read_sysfs_decimal(&path, "busnum").unwrap_or(0) as u8;
        let dev_num = read_sysfs_decimal(&path, "devnum").unwrap_or(0) as u8;
        let device_class = read_sysfs_hex(&path, "bDeviceClass").unwrap_or(0) as u8;

        devices.push(UsbDevice {
            vendor_id,
            product_id,
            vendor_name,
            product_name,
            bus_num,
            dev_num,
            device_class,
        });
    }

    Ok(devices)
}

fn read_sysfs_hex(path: &std::path::Path, attr: &str) -> Option<u16> {
    let value = std::fs::read_to_string(path.join(attr)).ok()?;
    u16::from_str_radix(value.trim(), 16).ok()
}

fn read_sysfs_decimal(path: &std::path::Path, attr: &str) -> Option<u32> {
    let value = std::fs::read_to_string(path.join(attr)).ok()?;
    value.trim().parse().ok()
}

fn read_sysfs_string(path: &std::path::Path, attr: &str) -> Option<String> {
    std::fs::read_to_string(path.join(attr))
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_device_display() {
        let device = UsbDevice {
            vendor_id: 0x046d,
            product_id: 0xc077,
            vendor_name: "Logitech".to_string(),
            product_name: "M105 Mouse".to_string(),
            bus_num: 1,
            dev_num: 3,
            device_class: 0,
        };

        assert_eq!(device.display_name(), "Logitech M105 Mouse");
        assert!(!device.is_hub());
    }

    #[test]
    fn test_qemu_args() {
        let device = UsbDevice {
            vendor_id: 0x046d,
            product_id: 0xc077,
            vendor_name: "Logitech".to_string(),
            product_name: "M105 Mouse".to_string(),
            bus_num: 1,
            dev_num: 3,
            device_class: 0,
        };

        let args = device.to_qemu_args();
        assert_eq!(args[0], "-device");
        assert!(args[1].contains("vendorid=0x046d"));
        assert!(args[1].contains("productid=0xc077"));
    }
}
