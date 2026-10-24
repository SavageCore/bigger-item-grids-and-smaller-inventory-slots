//! Per-feature on/off switches, read from nms-tweaks.toml next to the DLL.
//!
//! Format:
//! ```toml
//! [features]
//! intro_skip = true
//! mod_warning_skip = true
//! inventory_grid = true
//! popup_pages = true
//! ```
//! Missing file or keys mean all-on. Deliberately hand-parsed: no
//! dependencies, fully predictable behaviour inside DllMain.

use std::ffi::c_void;

pub struct Config {
    pub intro_skip: bool,
    pub mod_warning_skip: bool,
    pub inventory_grid: bool,
    pub popup_pages: bool,
}

/// Fallback location: the static mod folder, for manager installs where
/// the toml lands with the mod files instead of beside the DLL.
const MOD_DIR_NAME: &str = "Bigger Item Grids and Smaller Inventory Slots";

impl Config {
    fn all_on() -> Self {
        Self {
            intro_skip: true,
            mod_warning_skip: true,
            inventory_grid: true,
            popup_pages: true,
        }
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// <game>/GAMEDATA/MODS/<mod>/nms-tweaks.toml, derived from the host exe
/// living in <game>/Binaries. Used when a manager deploys the toml with
/// the mod files instead of beside the DLL.
fn read_mods_copy() -> Option<String> {
    use windows_sys::Win32::Foundation::MAX_PATH;
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameA;
    unsafe {
        let mut buf = [0u8; MAX_PATH as usize];
        let n = GetModuleFileNameA(std::ptr::null_mut(), buf.as_mut_ptr(), MAX_PATH as u32);
        if n == 0 {
            return None;
        }
        let host = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
        // host = <game>/Binaries/NMS.exe (separators may be / or \).
        // Strip twice: exe name, then the Binaries dir itself.
        let bin_end = host.rfind(['/', '\\'])?;
        let binaries = &host[..bin_end];
        let game_end = binaries.rfind(['/', '\\'])?;
        let game = &binaries[..game_end];
        let sep = if host.contains('\\') { "\\" } else { "/" };
        let path = format!(
            "{}{}GAMEDATA{}MODS{}{}{}{}",
            game, sep, sep, sep, MOD_DIR_NAME, sep, "nms-tweaks.toml"
        );
        std::fs::read_to_string(&path).ok()
    }
}

pub fn load(hinst: *mut c_void, log: &mut crate::log::Log) -> Config {
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameA;
    use windows_sys::Win32::Foundation::MAX_PATH;
    let mut cfg = Config::all_on();
    unsafe {
        let mut buf = [0u8; MAX_PATH as usize];
        let n = GetModuleFileNameA(hinst, buf.as_mut_ptr(), MAX_PATH as u32);
        if n == 0 {
            log.line("config: module path unknown, all features on");
            return cfg;
        }
        let mut path = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
        if let Some(sep) = path.rfind(['/', '\\']) {
            path.truncate(sep + 1);
        } else {
            path.clear();
        }
        path.push_str("nms-tweaks.toml");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => {
                log.line("config: nms-tweaks.toml loaded");
                t
            }
            Err(_) => match read_mods_copy() {
                Some(t) => {
                    log.line("config: nms-tweaks.toml loaded from MODS");
                    t
                }
                None => {
                    log.line("config: nms-tweaks.toml not found, all features on");
                    return cfg;
                }
            },
        };
        let mut in_features = false;
        for raw in text.lines() {
            let line = raw.split(['#', ';']).next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                in_features = line.eq_ignore_ascii_case("[features]");
                continue;
            }
            if !in_features {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let val = match parse_bool(v) {
                    Some(b) => b,
                    None => continue,
                };
                match k.trim().to_ascii_lowercase().as_str() {
                    "intro_skip" => cfg.intro_skip = val,
                    "mod_warning_skip" => cfg.mod_warning_skip = val,
                    "inventory_grid" => cfg.inventory_grid = val,
                    "popup_pages" => cfg.popup_pages = val,
                    _ => {}
                }
            }
        }
    }
    cfg
}
