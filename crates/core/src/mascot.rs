//! Ekspresi maskot Notchi (PRD §2.6).
//!
//! Modul ini hanya menentukan ekspresi dan parameter gerak; gerak berulang
//! (kedip, napas, ayun) dijalankan sebagai animasi compositor agar tidak
//! membangunkan thread aplikasi.

use crate::activity::ActivityKind;
use crate::island::AlertKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expression {
    /// Mata titik berkedip pelan, napas halus.
    Idle,
    /// Musik atau video diputar: mata terpejam, tubuh berayun.
    Listening,
    /// Timer berjalan: mata melirik angka hitung mundur.
    Watching,
    /// Mengisi daya: mata berbentuk kilat kecil.
    Charging,
    /// Baterai rendah: mata setengah terbuka, gerak lambat.
    Sleepy,
    /// Notifikasi atau alert: mata membesar, melompat sekali.
    Surprised,
    /// Layar penuh atau Reduce Motion: pose netral statis.
    Neutral,
}

/// Masukan untuk memilih ekspresi.
#[derive(Debug, Clone, Copy, Default)]
pub struct MascotContext<'a> {
    pub alert: Option<&'a AlertKind>,
    pub primary: Option<ActivityKind>,
    pub media_playing: bool,
    pub low_battery: bool,
    pub reduce_motion: bool,
}

pub fn expression_for(ctx: MascotContext<'_>) -> Expression {
    if let Some(alert) = ctx.alert {
        return match alert {
            AlertKind::Charging { .. } => Expression::Charging,
            AlertKind::LowBattery { .. } => Expression::Sleepy,
            AlertKind::Volume { .. } if ctx.media_playing => Expression::Listening,
            AlertKind::Volume { .. } => Expression::Idle,
            AlertKind::Unplugged { .. } | AlertKind::TimerDone => Expression::Surprised,
        };
    }
    match ctx.primary {
        Some(ActivityKind::Timer | ActivityKind::Stopwatch) => Expression::Watching,
        Some(ActivityKind::Media) if ctx.media_playing => Expression::Listening,
        _ if ctx.low_battery => Expression::Sleepy,
        _ => Expression::Idle,
    }
}

/// Parameter gerak yang diterjemahkan ke animasi compositor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// Skala napas (min, maks) dan periode dalam ms.
    pub breathe: Option<(f32, f32, u32)>,
    /// Periode kedip dalam ms.
    pub blink_every: Option<u32>,
    /// Ayunan tubuh (derajat, periode ms).
    pub sway: Option<(f32, u32)>,
    /// Melompat sekali dengan preset bouncy.
    pub hop: bool,
    /// Skala vertikal mata (1 = bulat penuh, kecil = terpejam).
    pub eye_open: f32,
    /// Skala ukuran mata.
    pub eye_scale: f32,
    /// Geser mata ke samping, dalam fraksi lebar tubuh (lirikan).
    pub eye_shift: f32,
    /// Mata digambar sebagai kilat.
    pub bolt_eyes: bool,
}

impl Motion {
    const STILL: Motion = Motion {
        breathe: None,
        blink_every: None,
        sway: None,
        hop: false,
        eye_open: 1.0,
        eye_scale: 1.0,
        eye_shift: 0.0,
        bolt_eyes: false,
    };
}

pub fn motion_for(expr: Expression, reduce_motion: bool) -> Motion {
    let m = match expr {
        Expression::Idle => Motion {
            breathe: Some((1.00, 1.03, 3_200)),
            blink_every: Some(4_500),
            ..Motion::STILL
        },
        Expression::Listening => {
            Motion { sway: Some((7.0, 1_400)), eye_open: 0.22, ..Motion::STILL }
        }
        Expression::Watching => Motion {
            breathe: Some((1.00, 1.02, 3_200)),
            blink_every: Some(5_000),
            eye_shift: 0.12,
            ..Motion::STILL
        },
        Expression::Charging => Motion { hop: true, bolt_eyes: true, ..Motion::STILL },
        Expression::Sleepy => Motion {
            breathe: Some((1.00, 1.02, 6_000)),
            eye_open: 0.45,
            ..Motion::STILL
        },
        Expression::Surprised => Motion { hop: true, eye_scale: 1.35, ..Motion::STILL },
        Expression::Neutral => Motion::STILL,
    };
    if reduce_motion {
        // Pose tetap dipertahankan, tetapi tanpa animasi apa pun.
        Motion { breathe: None, blink_every: None, sway: None, hop: false, ..m }
    } else {
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_mengalahkan_aktivitas() {
        let alert = AlertKind::Charging { percent: Some(40) };
        let ctx = MascotContext {
            alert: Some(&alert),
            primary: Some(ActivityKind::Media),
            media_playing: true,
            ..Default::default()
        };
        assert_eq!(expression_for(ctx), Expression::Charging);
    }

    #[test]
    fn media_hanya_saat_diputar() {
        let mut ctx = MascotContext { primary: Some(ActivityKind::Media), ..Default::default() };
        assert_eq!(expression_for(ctx), Expression::Idle);
        ctx.media_playing = true;
        assert_eq!(expression_for(ctx), Expression::Listening);
    }

    #[test]
    fn timer_melirik() {
        let ctx = MascotContext { primary: Some(ActivityKind::Timer), ..Default::default() };
        assert_eq!(expression_for(ctx), Expression::Watching);
    }

    #[test]
    fn baterai_rendah_mengantuk() {
        let ctx = MascotContext { low_battery: true, ..Default::default() };
        assert_eq!(expression_for(ctx), Expression::Sleepy);
    }

    #[test]
    fn reduce_motion_tanpa_animasi() {
        for e in [
            Expression::Idle,
            Expression::Listening,
            Expression::Watching,
            Expression::Charging,
            Expression::Sleepy,
            Expression::Surprised,
        ] {
            let m = motion_for(e, true);
            assert!(m.breathe.is_none() && m.blink_every.is_none() && m.sway.is_none() && !m.hop);
        }
    }

    #[test]
    fn napas_dalam_rentang_prd() {
        let (lo, hi, _) = motion_for(Expression::Idle, false).breathe.unwrap();
        assert_eq!((lo, hi), (1.00, 1.03));
    }
}
