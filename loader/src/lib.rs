//! nms-tweaks: single xinput9_1_0.dll proxy for No Man's Sky.
//!
//! Features (each independently toggleable via nms-tweaks.toml):
//! - intro_skip: skip startup logo wait + logo texture loading
//! - mod_warning_skip: skip the "mods detected" warning dialog
//! - inventory_grid: (phase 2) runtime slot size / visible-row override
//! - popup_pages: (phase 2) runtime items-per-page override
//!
//! The boot/logo/warning patches were reimplemented from scratch: site
//! locations were verified by disassembling the game's NMS.exe directly
//! (patterns confirmed unique against the installed build), and the patch
//! bytes are original designs (NOP a wait-branch, force a skip-branch,
//! retarget a dialog jmp). No third-party code is used in this phase.

use std::ffi::c_void;

mod config;
mod features;
mod image;
mod log;
mod patch;
mod proxy;

const DLL_PROCESS_ATTACH: u32 = 1;

/// Entry point. Runs synchronously on attach: forwarders first (the game
/// needs them immediately), then config, then patches, then the log.
/// Everything is fail-loud per feature; one broken signature never blocks
/// the others or the controller forwarding.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason != DLL_PROCESS_ATTACH {
        return 1;
    }
    // No thread notifications; we do all work inline on attach.
    proxy::disable_thread_calls(hinst);
    init(hinst);
    1
}

unsafe fn init(hinst: *mut c_void) {
    let mut log = log::Log::new("nms-tweaks");
    log.header(hinst);

    if !proxy::host_is_nms() {
        log.line("host is not NMS.exe: forwarding xinput only, patches skipped");
        proxy::load_real(&mut log);
        log.write_beside(hinst);
        return;
    }

    proxy::load_real(&mut log);

    let cfg = config::load(hinst, &mut log);
    log.line(&format!(
        "features: intro_skip={} mod_warning_skip={} inventory_grid={} popup_pages={}",
        cfg.intro_skip, cfg.mod_warning_skip, cfg.inventory_grid, cfg.popup_pages
    ));

    let mut done = 0;
    let mut total = 0;
    let mut run = |name: &str, enabled: bool, f: &dyn Fn(&mut log::Log) -> Result<String, String>| {
        if !enabled {
            log.line(&format!("{name}: disabled in config"));
            return;
        }
        total += 1;
        match f(&mut log) {
            Ok(detail) => {
                done += 1;
                log.line(&format!("{name}: applied {detail}"));
            }
            Err(e) => log.line(&format!("{name}: NOT APPLIED - {e}")),
        }
    };

    run(
        "logo-texture",
        cfg.intro_skip,
        &features::intro::logo_texture,
    );
    run("logo-wait", cfg.intro_skip, &features::intro::logo_wait);
    run(
        "mod-warning",
        cfg.mod_warning_skip,
        &features::intro::mod_warning,
    );
    run(
        "inventory-grid",
        cfg.inventory_grid,
        &features::inventory::inventory_grid,
    );
    run("popup-pages", cfg.popup_pages, &features::popup::popup_pages);

    log.line(&format!("result: {done}/{total} applied"));
    log.write_beside(hinst);
}
