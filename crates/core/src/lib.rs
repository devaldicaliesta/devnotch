//! Logika murni Notchi (PRD §6.2): tidak memegang handle Windows dan tidak
//! memakai `unsafe`, sehingga seluruhnya dapat diuji di Linux CI.
//!
//! Waktu selalu diberikan oleh pemanggil sebagai [`Ms`] (milidetik monoton),
//! jadi setiap modul deterministik dan mudah diuji.

#![forbid(unsafe_code)]

pub mod activity;
pub mod color;
pub mod config;
pub mod fmt;
pub mod geom;
pub mod hotkey;
pub mod i18n;
pub mod icon;
pub mod island;
pub mod layout;
pub mod mascot;
pub mod media;
pub mod power;
pub mod spring;
pub mod timer;

/// Milidetik monoton sejak titik acuan yang dipilih pemanggil.
pub type Ms = u64;
