//! Boot-flow patches, reimplemented from direct disassembly of NMS.exe.
//!
//! All three sites were verified against the installed game build before
//! baking the patterns: each pattern is unique across the host .text
//! (logo-texture, logo-wait) or disambiguated structurally (mod-warning).
//! Every patch is runtime-computed (jump targets re-derived from the
//! existing rel32), so no absolute address is baked in.
//!
//! - logo_texture: the logo loop skips loading a texture slot when its
//!   `je` is taken; forcing the jump treats every slot as empty, so the
//!   loop spins through without loading anything.
//! - logo_wait: the splash state machine waits on a `je` to a wait
//!   handler; NOPing it falls through into the normal loop continuation.
//! - mod_warning: each dialog case does call/call/jmp into a shared
//!   `test al,al` check; the mod-warning case is the one jumping
//!   backwards. Retargeting its jmp to the check's skip destination
//!   behaves exactly like "check returned false".

use crate::log::Log;
use crate::patch;

// cmp rbx,rdi; je ??; xor r13d,r13d; cmp [rbx],r13b; je ?? (21 bytes)
const LT_PAT: [u8; 21] = [
    0x48, 0x3b, 0xdf, 0x0f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x45, 0x33, 0xed, 0x44, 0x38, 0x2b,
    0x0f, 0x84, 0x00, 0x00, 0x00, 0x00,
];
const LT_MASK: [u8; 21] = [
    1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
];
const LT_OFF: usize = 15; // second je

// cmp ecx,5; je ??; cmp ecx,6; je ??; cmp [rip],r12b; je ?????? (23 bytes)
const LW_PAT: [u8; 23] = [
    0x83, 0xf9, 0x05, 0x74, 0x00, 0x83, 0xf9, 0x06, 0x74, 0x00, 0x44, 0x38, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0f, 0x84, 0x00, 0x00, 0x00, 0x00,
];
const LW_MASK: [u8; 23] = [
    1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
];
const LW_OFF: usize = 17; // the far je

// call ????????; mov rcx,rax; call ????????; jmp ???????? (18 bytes)
const MW_PAT: [u8; 18] = [
    0xe8, 0x00, 0x00, 0x00, 0x00, 0x48, 0x8b, 0xc8, 0xe8, 0x00, 0x00, 0x00, 0x00, 0xe9,
    0x00, 0x00, 0x00, 0x00,
];
const MW_MASK: [u8; 18] = [1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0];
const MW_OFF: usize = 13; // the jmp

const NOP6: [u8; 6] = [0x66, 0x0f, 0x1f, 0x44, 0x00, 0x00];

fn text() -> Result<(usize, usize), String> {
    patch::host_text()
}

fn unique(pat: &[u8], mask: &[u8]) -> Result<usize, String> {
    let (base, len) = text()?;
    let hits = patch::scan(base, len, pat, mask);
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err("pattern matched nothing (game logic changed?)".into()),
        n => Err(format!("{n} ambiguous candidates (refused for safety)")),
    }
}

/// Force the logo-texture skip-branch always taken:
/// `0f 84 rel32` -> `e9 rel32' 90` with the same target.
pub fn logo_texture(_log: &mut Log) -> Result<String, String> {
    let hit = unique(&LT_PAT, &LT_MASK)?;
    let at = hit + LT_OFF;
    let cur = patch::read_bytes(at, 6);
    if cur[0] != 0x0f || cur[1] != 0x84 {
        return Err("skip-branch is not je as expected".into());
    }
    let old_rel = patch::read_i32_le(at + 2) as isize;
    let target = (at + 6).wrapping_add(old_rel as usize);
    let new_rel = target.wrapping_sub(at + 5) as i32;
    let mut buf = [0u8; 6];
    buf[0] = 0xe9;
    buf[1..5].copy_from_slice(&new_rel.to_le_bytes());
    buf[5] = 0x90;
    patch::write_bytes(at, &buf, Some(&cur))?;
    Ok(format!("@ rva {} [pattern match]", patch::rva_of(at)))
}

/// NOP the splash-wait branch so flow falls through to loop continuation.
pub fn logo_wait(_log: &mut Log) -> Result<String, String> {
    let hit = unique(&LW_PAT, &LW_MASK)?;
    let at = hit + LW_OFF;
    let cur = patch::read_bytes(at, 6);
    if cur[0] != 0x0f || cur[1] != 0x84 {
        return Err("wait-branch is not je as expected".into());
    }
    patch::write_bytes(at, &NOP6, Some(&cur))?;
    Ok(format!("@ rva {} [pattern match]", patch::rva_of(at)))
}

/// Retarget the mod-warning case's jmp at the shared `test al,al` check's
/// skip destination (i.e. behave as "check returned false").
///
/// The bare pattern also matches sibling dialog cases; the mod-warning
/// case is the one jumping *backwards* into the shared check, so filter
/// to backward jumps whose target starts with `test al,al` (84 c0).
pub fn mod_warning(_log: &mut Log) -> Result<String, String> {
    let (base, len) = text()?;
    let hits = patch::scan(base, len, &MW_PAT, &MW_MASK);
    if hits.is_empty() {
        return Err("pattern matched nothing (game logic changed?)".into());
    }
    let mut cands = Vec::new();
    for h in hits {
        let at = h + MW_OFF;
        let cur = patch::read_bytes(at, 5);
        if cur[0] != 0xe9 {
            continue;
        }
        let rel = patch::read_i32_le(at + 1) as isize;
        if rel >= 0 {
            continue; // sibling cases jump forward; we want the backward one
        }
        let target = (at + 5).wrapping_add(rel as usize);
        let tb = patch::read_bytes(target, 8);
        // test al,al (84 c0) followed by je, either near (0f 84 + rel32)
        // or short (74 + rel8). Derive the check's skip destination.
        if tb.len() < 8 || tb[0] != 0x84 || tb[1] != 0xc0 {
            continue;
        }
        let skip = if tb[2] == 0x0f && tb[3] == 0x84 {
            let skip_rel = patch::read_i32_le(target + 4) as isize;
            (target + 8).wrapping_add(skip_rel as usize)
        } else if tb[2] == 0x74 {
            (target + 4).wrapping_add(tb[3] as i8 as isize as usize)
        } else {
            continue;
        };
        cands.push((at, skip));
    }
    match cands.len() {
        1 => {
            let (at, skip) = cands[0];
            let new_rel = skip.wrapping_sub(at + 5) as i32;
            let mut buf = [0u8; 5];
            buf[0] = 0xe9;
            buf[1..5].copy_from_slice(&new_rel.to_le_bytes());
            let cur = patch::read_bytes(at, 5);
            patch::write_bytes(at, &buf, Some(&cur))?;
            Ok(format!("@ rva {} [pattern match]", patch::rva_of(at)))
        }
        0 => Err("no backward-jump case into a test al,al check".into()),
        n => Err(format!("{n} ambiguous candidates (refused for safety)")),
    }
}
