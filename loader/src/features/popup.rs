//! Transfer/Craft items-per-page override (ported algorithm, original code).
//!
//! Must match the static UI: our bigger_item_grids_preset.json is 8x3, so
//! ITEMS = 24. If the preset ever changes, this constant must follow it.
//! All-or-none: every site verifies before anything is written.

use crate::image;
use crate::log::Log;
use crate::patch;

const ITEMS: u32 = 24;
const VANILLA_ITEMS: u32 = 8;

fn host_exe_path() -> Result<String, String> {
    use windows_sys::Win32::Foundation::MAX_PATH;
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameA;
    unsafe {
        let mut buf = [0u8; MAX_PATH as usize];
        let n = GetModuleFileNameA(std::ptr::null_mut(), buf.as_mut_ptr(), MAX_PATH as u32);
        if n == 0 {
            return Err("host path unknown".into());
        }
        Ok(String::from_utf8_lossy(&buf[..n as usize]).into_owned())
    }
}

pub fn popup_pages(_log: &mut Log) -> Result<String, String> {
    if ITEMS == VANILLA_ITEMS {
        return Ok("8 items per page, the game default. Nothing patched.".into());
    }
    let exe = host_exe_path()?;
    let img = image::FileImage::load(&exe)?;
    let plans =
        image::plan_all(&img, ITEMS).map_err(|e| format!("left vanilla: {e}"))?;

    // Verify everything against live memory first (all-or-none).
    unsafe {
        let base =
            windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(std::ptr::null_mut())
                as usize;
        if base == 0 {
            return Err("host base unknown".into());
        }
        for p in &plans {
            for (rva, old, _) in &p.patches {
                let va = base + rva;
                let cur =
                    std::slice::from_raw_parts(va as *const u8, old.len()).to_vec();
                if cur != *old {
                    return Err(format!(
                        "left vanilla: {} code at {:#x} did not match NMS.exe",
                        p.label, rva
                    ));
                }
            }
        }
        // All verified: write caves then patches.
        for p in &plans {
            let cave = base + p.cave_rva;
            patch::write_bytes(cave, &p.scale.to_le_bytes(), Some(&[0xCC; 4]))
                .map_err(|e| format!("cave write failed: {e}"))?;
            for (rva, old, new) in &p.patches {
                let va = base + rva;
                patch::write_bytes(va, new, Some(old))
                    .map_err(|e| format!("{}: {e}", p.label))?;
            }
        }
    }
    Ok(format!(
        "{ITEMS} items per page on Transfer Items and Craft Product"
    ))
}
