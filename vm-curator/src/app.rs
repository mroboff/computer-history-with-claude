use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::hardware::UsbDevice;
use crate::metadata::{AsciiArtStore, MetadataStore, OsInfo};
use crate::vm::{
    discover_vms, group_vms_by_category, BootMode, DiscoveredVm, LaunchOptions, Snapshot,
};

/// Application screens/views
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Main VM list
    MainMenu,
    /// VM management options
    Management,
    /// Configuration view
    Configuration,
    /// Detailed info (history, blurbs)
    DetailedInfo,
    /// Snapshot management
    Snapshots,
    /// Boot options
    BootOptions,
    /// USB device selection
    UsbDevices,
    /// Confirmation dialog
    Confirm(ConfirmAction),
    /// Help screen
    Help,
    /// Search/filter
    Search,
}

/// Actions that need confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    LaunchVm,
    ResetVm,
    DeleteVm,
    DeleteSnapshot(String),
    RestoreSnapshot(String),
}

/// Input mode for text entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Application state
pub struct App {
    /// Current screen
    pub screen: Screen,
    /// Screen history for back navigation
    pub screen_stack: Vec<Screen>,
    /// Application configuration
    pub config: Config,
    /// Discovered VMs
    pub vms: Vec<DiscoveredVm>,
    /// Currently selected VM index
    pub selected_vm: usize,
    /// OS metadata store
    pub metadata: MetadataStore,
    /// ASCII art store
    pub ascii_art: AsciiArtStore,
    /// Snapshots for current VM (cached)
    pub snapshots: Vec<Snapshot>,
    /// Selected snapshot index
    pub selected_snapshot: usize,
    /// USB devices (cached)
    pub usb_devices: Vec<UsbDevice>,
    /// Selected USB devices for passthrough
    pub selected_usb_devices: Vec<usize>,
    /// Selected management menu item
    pub selected_menu_item: usize,
    /// Current boot mode
    pub boot_mode: BootMode,
    /// Search query
    pub search_query: String,
    /// Input mode
    pub input_mode: InputMode,
    /// Filtered VM indices (for search)
    pub filtered_indices: Vec<usize>,
    /// Status message
    pub status_message: Option<String>,
    /// Whether the app should quit
    pub should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Result<Self> {
        // Discover VMs
        let vms = discover_vms(&config.vm_library_path)?;

        // Load metadata
        let mut metadata = MetadataStore::load_embedded();
        if let Ok(user_metadata) = MetadataStore::load_from_dir(&config.metadata_path) {
            metadata.merge(user_metadata);
        }

        // Load ASCII art
        let mut ascii_art = AsciiArtStore::load_embedded();
        let user_art = AsciiArtStore::load_from_dir(&config.ascii_art_path);
        ascii_art.merge(user_art);

        let filtered_indices: Vec<usize> = (0..vms.len()).collect();

        Ok(Self {
            screen: Screen::MainMenu,
            screen_stack: Vec::new(),
            config,
            vms,
            selected_vm: 0,
            metadata,
            ascii_art,
            snapshots: Vec::new(),
            selected_snapshot: 0,
            usb_devices: Vec::new(),
            selected_usb_devices: Vec::new(),
            selected_menu_item: 0,
            boot_mode: BootMode::Normal,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            filtered_indices,
            status_message: None,
            should_quit: false,
        })
    }

    /// Get the currently selected VM
    pub fn selected_vm(&self) -> Option<&DiscoveredVm> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let actual_idx = self.filtered_indices.get(self.selected_vm)?;
        self.vms.get(*actual_idx)
    }

    /// Get OS info for the selected VM
    pub fn selected_vm_info(&self) -> Option<OsInfo> {
        let vm = self.selected_vm()?;
        self.metadata
            .get(&vm.id)
            .cloned()
            .or_else(|| Some(crate::metadata::default_os_info(&vm.id)))
    }

    /// Get ASCII art for the selected VM
    pub fn selected_vm_ascii(&self) -> &str {
        self.selected_vm()
            .map(|vm| self.ascii_art.get_or_fallback(&vm.id))
            .unwrap_or("")
    }

    /// Navigate to a new screen
    pub fn push_screen(&mut self, screen: Screen) {
        self.screen_stack.push(self.screen.clone());
        self.screen = screen;
        self.selected_menu_item = 0;
    }

    /// Go back to the previous screen
    pub fn pop_screen(&mut self) {
        if let Some(prev) = self.screen_stack.pop() {
            self.screen = prev;
        }
    }

    /// Move selection up in VM list
    pub fn select_prev(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected_vm > 0 {
            self.selected_vm -= 1;
        }
    }

    /// Move selection down in VM list
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected_vm < self.filtered_indices.len() - 1 {
            self.selected_vm += 1;
        }
    }

    /// Move selection up in menu
    pub fn menu_prev(&mut self) {
        if self.selected_menu_item > 0 {
            self.selected_menu_item -= 1;
        }
    }

    /// Move selection down in menu
    pub fn menu_next(&mut self, max_items: usize) {
        if self.selected_menu_item < max_items.saturating_sub(1) {
            self.selected_menu_item += 1;
        }
    }

    /// Update search filter
    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.vms.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = self
                .vms
                .iter()
                .enumerate()
                .filter(|(_, vm)| {
                    vm.display_name().to_lowercase().contains(&query)
                        || vm.id.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_vm >= self.filtered_indices.len() {
            self.selected_vm = self.filtered_indices.len().saturating_sub(1);
        }
    }

    /// Refresh VM list
    pub fn refresh_vms(&mut self) -> Result<()> {
        self.vms = discover_vms(&self.config.vm_library_path)?;
        self.update_filter();
        Ok(())
    }

    /// Load snapshots for the current VM
    pub fn load_snapshots(&mut self) -> Result<()> {
        self.snapshots.clear();
        self.selected_snapshot = 0;

        if let Some(vm) = self.selected_vm() {
            if let Some(disk) = vm.config.primary_disk() {
                if disk.format.supports_snapshots() {
                    self.snapshots = crate::vm::list_snapshots(&disk.path)?;
                }
            }
        }

        Ok(())
    }

    /// Load USB devices
    pub fn load_usb_devices(&mut self) -> Result<()> {
        self.usb_devices = crate::hardware::enumerate_usb_devices()?;
        self.selected_usb_devices.clear();
        Ok(())
    }

    /// Toggle USB device selection
    pub fn toggle_usb_device(&mut self, index: usize) {
        if let Some(pos) = self.selected_usb_devices.iter().position(|&i| i == index) {
            self.selected_usb_devices.remove(pos);
        } else {
            self.selected_usb_devices.push(index);
        }
    }

    /// Get launch options based on current state
    pub fn get_launch_options(&self) -> LaunchOptions {
        let usb_devices = self
            .selected_usb_devices
            .iter()
            .filter_map(|&i| self.usb_devices.get(i))
            .map(|d| crate::vm::UsbPassthrough {
                vendor_id: d.vendor_id,
                product_id: d.product_id,
            })
            .collect();

        LaunchOptions {
            boot_mode: self.boot_mode.clone(),
            extra_args: Vec::new(),
            usb_devices,
        }
    }

    /// Set a status message
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Get grouped VMs for display
    pub fn grouped_vms(&self) -> Vec<(&'static str, Vec<&DiscoveredVm>)> {
        group_vms_by_category(&self.vms)
    }
}
