#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]

mod arch {
    use bevy::input::keyboard::ScanCode;

    // Note: these aren't actually scan codes; they are just ASCII.
    // This is a Bevy shortcoming, and will be fixed in the next major
    // release. See https://github.com/bevyengine/bevy/pull/10702

    pub const W: ScanCode = ScanCode(b'W' as u32);
    pub const S: ScanCode = ScanCode(b'S' as u32);
    pub const A: ScanCode = ScanCode(b'A' as u32);
    pub const D: ScanCode = ScanCode(b'D' as u32);

    pub const UP: ScanCode = ScanCode(0x26);
    pub const DOWN: ScanCode = ScanCode(0x28);
    pub const LEFT: ScanCode = ScanCode(0x27);
    pub const RIGHT: ScanCode = ScanCode(0x25);

    pub const SPACE: ScanCode = ScanCode(0x20);
}

#[cfg(not(target_arch = "wasm32"))]
mod arch {
    use bevy::input::keyboard::ScanCode;

    pub const W: ScanCode = ScanCode(0x11);
    pub const S: ScanCode = ScanCode(0x1f);
    pub const A: ScanCode = ScanCode(0x1e);
    pub const D: ScanCode = ScanCode(0x20);

    pub const UP: ScanCode = ScanCode(0x67);
    pub const DOWN: ScanCode = ScanCode(0x6c);
    pub const LEFT: ScanCode = ScanCode(0x6a);
    pub const RIGHT: ScanCode = ScanCode(0x69);

    pub const SPACE: ScanCode = ScanCode(0x39);
}

pub use arch::*;
