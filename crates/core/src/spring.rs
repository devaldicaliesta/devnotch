//! Fisika pegas (PRD §2.3).
//!
//! Preset memakai semantik `duration`/`bounce` lalu dikonversi ke pegas
//! bermassa 1: `k = (2π/d)²`, `c = 4π(1−b)/d`. Compositor Windows menerima
//! parameter yang sama lewat `DampingRatio = 1 − bounce` dan `Period = duration`.
//! Integrator CPU di sini dipakai untuk pengujian dan untuk memperkirakan
//! kapan animasi selesai; animasi sebenarnya berjalan di compositor.

use core::f32::consts::PI;

/// Parameter pegas dalam bentuk yang mudah dibaca desainer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringPreset {
    /// Durasi persepsi dalam detik.
    pub duration: f32,
    /// 0 = tanpa pantulan, mendekati 1 = sangat memantul.
    pub bounce: f32,
}

impl SpringPreset {
    /// Pindah konten dalam kartu, fade.
    pub const SMOOTH: Self = Self::new(0.50, 0.00);
    /// Magnifikasi Dock, hover.
    pub const SNAPPY: Self = Self::new(0.40, 0.15);
    /// Island mengembang/mengecil, bounce ikon.
    pub const BOUNCY: Self = Self::new(0.50, 0.30);

    pub const fn new(duration: f32, bounce: f32) -> Self {
        Self { duration, bounce }
    }

    /// Kekakuan `k` untuk massa 1.
    pub fn stiffness(self) -> f32 {
        let w = 2.0 * PI / self.duration;
        w * w
    }

    /// Redaman `c` untuk massa 1.
    pub fn damping(self) -> f32 {
        4.0 * PI * (1.0 - self.bounce) / self.duration
    }

    /// Rasio redaman ζ = c / (2√k). Untuk preset ini ζ = 1 − bounce.
    pub fn damping_ratio(self) -> f32 {
        self.damping() / (2.0 * self.stiffness().sqrt())
    }

    /// Perkiraan waktu sampai simpangan tetap di bawah `tolerance`
    /// (relatif terhadap jarak awal), dihitung dengan simulasi.
    pub fn settle_time(self, tolerance: f32) -> f32 {
        let mut s = Spring::new(self, 0.0);
        s.set_target(1.0);
        let dt = 1.0 / 1000.0;
        let mut t = 0.0;
        let mut last_outside = 0.0;
        while t < 10.0 {
            s.step(dt);
            t += dt;
            if (s.value - 1.0).abs() > tolerance {
                last_outside = t;
            }
        }
        last_outside
    }
}

/// Pegas satu dimensi yang diintegrasikan di CPU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    pub preset: SpringPreset,
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
}

impl Spring {
    pub fn new(preset: SpringPreset, value: f32) -> Self {
        Self { preset, value, velocity: 0.0, target: value }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Satu langkah integrasi semi-implisit Euler.
    pub fn step(&mut self, dt: f32) {
        let k = self.preset.stiffness();
        let c = self.preset.damping();
        let force = -k * (self.value - self.target) - c * self.velocity;
        self.velocity += force * dt;
        self.value += self.velocity * dt;
    }

    pub fn is_settled(&self, epsilon: f32) -> bool {
        (self.value - self.target).abs() < epsilon && self.velocity.abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rumus_sesuai_prd() {
        let p = SpringPreset::new(0.5, 0.3);
        let w = 2.0 * PI / 0.5;
        assert!((p.stiffness() - w * w).abs() < 1e-3);
        assert!((p.damping() - 4.0 * PI * 0.7 / 0.5).abs() < 1e-4);
    }

    #[test]
    fn rasio_redaman_sama_dengan_satu_dikurangi_bounce() {
        for p in [SpringPreset::SMOOTH, SpringPreset::SNAPPY, SpringPreset::BOUNCY] {
            assert!((p.damping_ratio() - (1.0 - p.bounce)).abs() < 1e-5, "{p:?}");
        }
    }

    #[test]
    fn bouncy_selesai_dalam_500_ms() {
        // Kriteria penerimaan §3.5: animasi settle ≤ 500 ms (toleransi 2%).
        let t = SpringPreset::BOUNCY.settle_time(0.02);
        assert!(t <= 0.5, "settle {t} s");
    }

    #[test]
    fn smooth_tidak_melampaui_target() {
        let mut s = Spring::new(SpringPreset::SMOOTH, 0.0);
        s.set_target(1.0);
        for _ in 0..2000 {
            s.step(0.001);
            assert!(s.value <= 1.0 + 1e-4);
        }
        assert!(s.is_settled(1e-3));
    }

    #[test]
    fn bouncy_memantul_sekali() {
        let mut s = Spring::new(SpringPreset::BOUNCY, 0.0);
        s.set_target(1.0);
        let mut max: f32 = 0.0;
        for _ in 0..2000 {
            s.step(0.001);
            max = max.max(s.value);
        }
        assert!(max > 1.0, "harus ada overshoot");
        assert!(max < 1.15, "overshoot terlalu besar: {max}");
    }
}
