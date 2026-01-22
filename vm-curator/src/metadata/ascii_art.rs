use std::collections::HashMap;
use std::path::Path;

/// ASCII art storage and loading
#[derive(Debug, Clone, Default)]
pub struct AsciiArtStore {
    pub art: HashMap<String, String>,
}

impl AsciiArtStore {
    /// Load ASCII art from a directory
    pub fn load_from_dir(dir: &Path) -> Self {
        let mut store = Self::default();

        if !dir.exists() {
            return store;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "txt" || e == "ascii").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let id = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        store.art.insert(id, content);
                    }
                }
            }
        }

        store
    }

    /// Load embedded ASCII art
    pub fn load_embedded() -> Self {
        let mut store = Self::default();

        // Windows logos
        store.art.insert("windows-95".to_string(), WINDOWS_95_LOGO.to_string());
        store.art.insert("windows-98".to_string(), WINDOWS_98_LOGO.to_string());
        store.art.insert("windows-me".to_string(), WINDOWS_ME_LOGO.to_string());
        store.art.insert("windows-2000".to_string(), WINDOWS_2000_LOGO.to_string());
        store.art.insert("windows-xp".to_string(), WINDOWS_XP_LOGO.to_string());
        store.art.insert("windows-vista".to_string(), WINDOWS_VISTA_LOGO.to_string());
        store.art.insert("windows-7".to_string(), WINDOWS_7_LOGO.to_string());
        store.art.insert("windows-10".to_string(), WINDOWS_10_LOGO.to_string());
        store.art.insert("windows-11".to_string(), WINDOWS_11_LOGO.to_string());
        store.art.insert("dos".to_string(), DOS_LOGO.to_string());
        store.art.insert("ms-dos".to_string(), DOS_LOGO.to_string());

        // Mac logos
        store.art.insert("mac-system7".to_string(), MAC_CLASSIC_LOGO.to_string());
        store.art.insert("mac-os9".to_string(), MAC_OS9_LOGO.to_string());
        store.art.insert("mac-osx-tiger".to_string(), MAC_OSX_LOGO.to_string());
        store.art.insert("mac-osx-leopard".to_string(), MAC_OSX_LOGO.to_string());

        // Linux logos
        store.art.insert("linux-fedora".to_string(), FEDORA_LOGO.to_string());
        store.art.insert("linux".to_string(), TUX_LOGO.to_string());

        store
    }

    /// Get ASCII art for a VM ID
    pub fn get(&self, id: &str) -> Option<&str> {
        self.art.get(id).map(|s| s.as_str())
    }

    /// Get ASCII art or a fallback based on OS family
    pub fn get_or_fallback(&self, id: &str) -> &str {
        if let Some(art) = self.get(id) {
            return art;
        }

        // Try to find a fallback based on the name
        let id_lower = id.to_lowercase();
        if id_lower.contains("windows") {
            return WINDOWS_GENERIC_LOGO;
        } else if id_lower.contains("mac") || id_lower.contains("osx") {
            return MAC_GENERIC_LOGO;
        } else if id_lower.contains("linux") || id_lower.contains("fedora") || id_lower.contains("ubuntu") {
            return TUX_LOGO;
        } else if id_lower.contains("dos") {
            return DOS_LOGO;
        }

        GENERIC_VM_LOGO
    }

    /// Merge user overrides
    pub fn merge(&mut self, overrides: AsciiArtStore) {
        for (id, art) in overrides.art {
            self.art.insert(id, art);
        }
    }
}

// Embedded ASCII art logos

const WINDOWS_95_LOGO: &str = r#"
 __        ___           _                     ___  ____
 \ \      / (_)_ __   __| | _____      _____  / _ \| ___|
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|| (_) |___ \
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \ \__, |___) |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/   /_/|____/
"#;

const WINDOWS_98_LOGO: &str = r#"
 __        ___           _                     ___   ___
 \ \      / (_)_ __   __| | _____      _____  / _ \ ( _ )
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|| (_) |/ _ \
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \ \__, | (_) |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/   /_/ \___/
"#;

const WINDOWS_ME_LOGO: &str = r#"
 __        ___           _                     __  __ _____
 \ \      / (_)_ __   __| | _____      _____  |  \/  | ____|
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __| | |\/| |  _|
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \ | |  | | |___
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/ |_|  |_|_____|
"#;

const WINDOWS_2000_LOGO: &str = r#"
 __        ___           _                     ____   ___   ___   ___
 \ \      / (_)_ __   __| | _____      _____  |___ \ / _ \ / _ \ / _ \
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|   __) | | | | | | | | | |
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \  / __/| |_| | |_| | |_| |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/ |_____|\___/ \___/ \___/
"#;

const WINDOWS_XP_LOGO: &str = r#"
 __        ___           _                    __  ______
 \ \      / (_)_ __   __| | _____      _____  \ \/ /  _ \
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|  \  /| |_) |
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \  /  \|  __/
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/ /_/\_\_|
"#;

const WINDOWS_VISTA_LOGO: &str = r#"
 __        ___           _                    __     ___     _
 \ \      / (_)_ __   __| | _____      _____  \ \   / (_)___| |_ __ _
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|  \ \ / /| / __| __/ _` |
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \   \ V / | \__ \ || (_| |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/    \_/  |_|___/\__\__,_|
"#;

const WINDOWS_7_LOGO: &str = r#"
 __        ___           _                    _____
 \ \      / (_)_ __   __| | _____      _____  |___  |
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|    / /
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \   / /
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/  /_/
"#;

const WINDOWS_10_LOGO: &str = r#"
 __        ___           _                    _  ___
 \ \      / (_)_ __   __| | _____      _____  / |/ _ \
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __| | | | | |
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \ | | |_| |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/ |_|\___/
"#;

const WINDOWS_11_LOGO: &str = r#"
 __        ___           _                    _ _
 \ \      / (_)_ __   __| | _____      _____  / / |
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __| | | |
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \ | | |
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/ |_|_|
"#;

const WINDOWS_GENERIC_LOGO: &str = r#"
 __        ___           _
 \ \      / (_)_ __   __| | _____      _____
  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / __|
   \ V  V / | | | | | (_| | (_) \ V  V /\__ \
    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/ |___/
"#;

const DOS_LOGO: &str = r#"
  __  __ ____        ____   ___  ____
 |  \/  / ___|      |  _ \ / _ \/ ___|
 | |\/| \___ \ _____| | | | | | \___ \
 | |  | |___) |_____| |_| | |_| |___) |
 |_|  |_|____/      |____/ \___/|____/
"#;

const MAC_CLASSIC_LOGO: &str = r#"
      _____
     /     \
    | () () |
    |   ^   |
    |  ___  |
    |_______|
   Macintosh
"#;

const MAC_OS9_LOGO: &str = r#"
    ___  ____    ___
   / _ \/ ___|  / _ \
  | | | \___ \ | (_) |
  | |_| |___) | \__, |
   \___/|____/    /_/
     Mac OS 9
"#;

const MAC_OSX_LOGO: &str = r#"
        .:'
    __ :'__
 .'`  `-'  ``.
:          .-'
:         :
 :         `-;
  `.__.-.__.'
   Mac OS X
"#;

const MAC_GENERIC_LOGO: &str = r#"
      _____
     /     \
    |       |
    |   @   |
    |_______|
   Macintosh
"#;

const TUX_LOGO: &str = r#"
       .--.
      |o_o |
      |:_/ |
     //   \ \
    (|     | )
   /'\_   _/`\
   \___)=(___/
"#;

const FEDORA_LOGO: &str = r#"
        _____
       /     \
      |  f    |
      |   e   |
      |    d  |
      |     o |
      |      r|
       \_____a|
"#;

const GENERIC_VM_LOGO: &str = r#"
   +----------+
   |  ______  |
   | |      | |
   | | QEMU | |
   | |______| |
   |__________|
     Virtual
"#;
