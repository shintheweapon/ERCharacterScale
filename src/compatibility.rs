//! Exact private bindings supplement fsrs. Object offsets are never RVA-mapped.
//! Expected code is derived from retained evidence, never from current memory.
use std::sync::atomic::{AtomicUsize, Ordering};
#[path = "compatibility_data.rs"]
mod data;

static IMAGE_BASE: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static OWNED_WW272: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn is_ww272() -> bool {
    #[cfg(test)]
    if OWNED_WW272.load(Ordering::Acquire) {
        return true;
    }
    eldenring::is_ww272()
}

pub(crate) fn set_image_base(base: usize) {
    IMAGE_BASE.store(base, Ordering::Release);
}

pub(crate) fn rva(legacy: usize) -> usize {
    if !is_ww272() {
        return legacy;
    }
    data::BINDINGS
        .binary_search_by_key(&legacy, |v| v.0)
        .ok()
        .map_or(legacy, |i| data::BINDINGS[i].1)
}

/// Existing type/route selectors use stable logical IDs. New runtime pointers
/// are converted only through this exact inverse table, not a section delta.
pub(crate) fn canonical_rva(current: usize) -> usize {
    if !is_ww272() {
        return current;
    }
    data::INVERSE
        .binary_search_by_key(&current, |v| v.0)
        .ok()
        .map_or(current, |i| data::INVERSE[i].1)
}

pub(crate) fn matches_code(legacy: usize, actual: &[u8], expected: &[u8]) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    if !is_ww272() {
        return actual == expected;
    }
    actual
        .iter()
        .zip(expected)
        .enumerate()
        .all(|(i, (&actual, &expected))| {
            match data::ADDRESS_BYTES.binary_search_by_key(&(legacy + i), |v| v.0) {
                Ok(at) => {
                    let (_, old, new) = data::ADDRESS_BYTES[at];
                    expected == old && actual == new
                }
                Err(_) => actual == expected,
            }
        })
}

pub(crate) fn matches_address(address: usize, actual: &[u8], expected: &[u8]) -> bool {
    let base = IMAGE_BASE.load(Ordering::Acquire);
    if base == 0 || address < base || address - base >= image_size() {
        return actual == expected;
    }
    matches_code(canonical_rva(address - base), actual, expected)
}

pub(crate) fn image_size() -> usize {
    if is_ww272() { 0x5dfd600 } else { 0x5e0da00 }
}
pub(crate) fn timestamp() -> u32 {
    if is_ww272() { 0x6aabf7e7 } else { 0x6a96b418 }
}
pub(crate) fn entrypoint() -> u32 {
    if is_ww272() { 0x24ff890 } else { 0x24fe160 }
}

#[cfg(test)]
pub(crate) struct OwnedImageProfile;
#[cfg(test)]
impl Drop for OwnedImageProfile {
    fn drop(&mut self) {
        OWNED_WW272.store(false, Ordering::Release);
        set_image_base(0);
    }
}
#[cfg(test)]
pub(crate) fn owned_image_profile(ww272: bool) -> OwnedImageProfile {
    OWNED_WW272.store(ww272, Ordering::Release);
    OwnedImageProfile
}
