//! Pembungkus aman untuk Win32, WinRT, Composition, dan Direct2D (PRD §6.2).
//!
//! Ini **satu-satunya** crate Notchi yang boleh memakai `unsafe`; setiap blok
//! `unsafe` diberi komentar `SAFETY:`. Crate fitur hanya memakai API aman
//! yang diekspor dari sini.
//!
//! API WinRT Composition sendiri sudah aman di crate `windows`, jadi tipe-tipe
//! itu diekspor ulang lewat [`comp`] agar crate fitur tidak perlu bergantung
//! langsung pada `windows`.

#![cfg(windows)]

pub mod audio;
pub mod comp;
pub mod fswatch;
pub mod gfx;
pub mod media;
pub mod power;
pub mod shell;
pub mod sys;
pub mod window;

pub use windows_core::{Error, HSTRING, Result};

use std::sync::OnceLock;
use std::time::Instant;

/// Jam monoton bersama untuk seluruh proses, dalam milidetik.
///
/// Thread media dan thread UI memakai jam yang sama sehingga posisi lagu
/// dapat diekstrapolasi dengan benar.
pub fn now_ms() -> notchi_core::Ms {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as notchi_core::Ms
}
