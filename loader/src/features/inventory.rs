//! Runtime inventory grid override (ported algorithm, original code).
//!
//! The static MBINs change slot visuals and box sizes, but the game's
//! DoInventorySlots renderer is called with vanilla metrics (90px slots,
//! 5 cargo / 3 tech visible rows). Extra rows are drawn but inactive
//! (greyed). This detours the renderer and overrides, per call:
//! slot 90 -> 63, cargo rows 5 -> 7, tech rows 3 -> 4.
//!
//! Guards (fail-safe: any mismatch calls through untouched):
//! - only when the game passes vanilla slot_width 90,
//! - only for vanilla render_heights 5 (cargo) / 3 (tech),
//! - only when the layer element is our scaled INVENTORY panel
//!   (ID "INVENTORY", width ~= 724.5).
//!
//! Must match files/UI preset scale 0.7: slot 63, pitch 69,
//! panel_w 724.5, cargo 7 rows, tech 4 rows.

use retour::RawDetour;

use crate::log::Log;
use crate::patch;

// DoInventorySlots hook signature, verified unique on the target build.
const HOOK_PAT: [u8; 19] = [
    0x48, 0x85, 0xD2, 0x0F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xC4, 0x4C, 0x89,
    0x48, 0x00, 0x4C, 0x89, 0x40,
];
const HOOK_MASK: [u8; 19] = [1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1];

const VANILLA_SLOT: i32 = 90;
const SLOT: i32 = 63;
const CARGO_ROWS: i32 = 7;
const TECH_ROWS: i32 = 4;
const PANEL_W: f64 = 724.5;

// UI element struct offsets (GcNGuiElementData).
const ELEMENT_DATA: usize = 0x48;
const DATA_ID: usize = 0x48;
const LAYOUT_W: usize = 0x34;
const MIN_ADDR: usize = 0x10000;

type DoSlotsFn = unsafe extern "system" fn(
    u64,
    u64,
    u64,
    u64,
    i32,
    i32,
    u8,
    u64,
    i32,
    i32,
    u8,
    i32,
    i32,
    u8,
    f32,
) -> u64;

static mut TRAMPOLINE: *const () = std::ptr::null();
static mut DETOUR: Option<RawDetour> = None;

fn read_ptr(addr: usize) -> Option<usize> {
    if addr < MIN_ADDR {
        return None;
    }
    unsafe {
        let p = *(addr as *const usize);
        if p < MIN_ADDR {
            None
        } else {
            Some(p)
        }
    }
}

fn read_ascii16(addr: usize) -> Option<String> {
    if addr < MIN_ADDR {
        return None;
    }
    unsafe {
        let raw = std::slice::from_raw_parts(addr as *const u8, 16);
        let end = raw.iter().position(|&b| b == 0).unwrap_or(16);
        let s = std::str::from_utf8(&raw[..end]).ok()?;
        if s.is_empty() || !s.chars().all(|c| c.is_ascii() && !c.is_control()) {
            return None;
        }
        Some(s.to_string())
    }
}

fn read_f32(addr: usize) -> Option<f32> {
    if addr < MIN_ADDR {
        return None;
    }
    unsafe {
        let v = *(addr as *const f32);
        if v.is_finite() {
            Some(v)
        } else {
            None
        }
    }
}

/// (rows, ) if this call draws our scaled cargo/tech grid, else None.
fn override_rows(slot_width: i32, render_height: i32, layer: u64) -> Option<i32> {
    if slot_width != VANILLA_SLOT {
        return None;
    }
    let rows = match render_height {
        5 => CARGO_ROWS,
        3 => TECH_ROWS,
        _ => return None,
    };
    if layer == 0 {
        return None;
    }
    let data = read_ptr(layer as usize + ELEMENT_DATA)?;
    let ident = read_ascii16(data + DATA_ID)?;
    if ident != "INVENTORY" {
        return None;
    }
    let w = read_f32(data + LAYOUT_W)? as f64;
    if (w - PANEL_W).abs() > 0.5 {
        return None;
    }
    Some(rows)
}

unsafe extern "system" fn detour(
    page: u64,
    layer: u64,
    inventory: u64,
    empty_slot_actions: u64,
    slots_wide: i32,
    popup_actions: i32,
    disabled: u8,
    min_index: u64,
    slot_width: i32,
    slot_height: i32,
    can_hide_slots: u8,
    render_width: i32,
    render_height: i32,
    compare_special: u8,
    scroll_offset: f32,
) -> u64 {
    if TRAMPOLINE.is_null() {
        return 0;
    }
    let orig: DoSlotsFn = std::mem::transmute(TRAMPOLINE);
    match override_rows(slot_width, render_height, layer) {
        Some(rows) => orig(
            page,
            layer,
            inventory,
            empty_slot_actions,
            slots_wide,
            popup_actions,
            disabled,
            min_index,
            SLOT,
            SLOT,
            can_hide_slots,
            render_width,
            rows,
            compare_special,
            scroll_offset,
        ),
        None => orig(
            page,
            layer,
            inventory,
            empty_slot_actions,
            slots_wide,
            popup_actions,
            disabled,
            min_index,
            slot_width,
            slot_height,
            can_hide_slots,
            render_width,
            render_height,
            compare_special,
            scroll_offset,
        ),
    }
}

pub fn inventory_grid(_log: &mut Log) -> Result<String, String> {
    let (base, len) = patch::host_text()?;
    let hits = patch::scan(base, len, &HOOK_PAT, &HOOK_MASK);
    if hits.len() != 1 {
        if hits.is_empty() {
            return Err("slot renderer hook pattern matched nothing (game update?)".into());
        }
        return Err(format!(
            "{} ambiguous hook candidates (refused for safety)",
            hits.len()
        ));
    }
    let target = hits[0];
    unsafe {
        let d = RawDetour::new(target as *const (), detour as *const ())
            .map_err(|e| format!("detour create failed: {e:?}"))?;
        d.enable()
            .map_err(|e| format!("detour enable failed: {e:?}"))?;
        TRAMPOLINE = d.trampoline();
        DETOUR = Some(d);
    }
    Ok(format!(
        "slot renderer hooked @ rva {} [pattern match]",
        patch::rva_of(target)
    ))
}
