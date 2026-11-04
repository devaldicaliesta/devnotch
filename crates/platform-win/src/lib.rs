#![cfg(windows)]

pub mod audio;
pub mod bluetooth;
pub mod brightness;
pub mod comp;
pub mod dragdrop;
pub mod fswatch;
pub mod gfx;
pub mod media;
pub mod net;
pub mod power;
pub mod privacy;
pub mod shell;
pub mod sys;
pub mod window;

pub use windows_core::{Error, HSTRING, Result};

use std::sync::OnceLock;
use std::time::Instant;

pub fn now_ms() -> notchi_core::Ms {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as notchi_core::Ms
}
