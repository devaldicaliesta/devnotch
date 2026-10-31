//! Rasterizer kecil untuk ikon Notchi (tray dan berkas ikon).
//!
//! Bentuknya sama dengan maskot di Island: tubuh bulat lunak berwarna skin
//! dengan dua mata titik. Antialiasing memakai supersampling 4×4.

use crate::color::{Color, Skin};

const EYE: Color = Color::rgb(0x141414);

/// Jarak bertanda ke persegi bersudut bulat yang berpusat di `(cx, cy)`.
fn rounded_rect_sdf(x: f32, y: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (x - cx).abs() - (hw - r);
    let qy = (y - cy).abs() - (hh - r);
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

fn ellipse_inside(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> bool {
    let dx = (x - cx) / rx;
    let dy = (y - cy) / ry;
    dx * dx + dy * dy <= 1.0
}

/// Ikon maskot berukuran `size × size` dalam RGBA lurus (tidak premultiplied).
pub fn mascot_rgba(size: u32, skin: Skin) -> Vec<u8> {
    let s = size as f32;
    let body = skin.color();
    let (bcx, bcy, bhw, bhh, br) = (s * 0.5, s * 0.52, s * 0.46, s * 0.42, s * 0.40);
    let eyes = [(s * 0.36, s * 0.50), (s * 0.64, s * 0.50)];
    let (erx, ery) = (s * 0.075, s * 0.10);
    const N: usize = 4;
    let mut out = vec![0u8; (size * size * 4) as usize];
    for py in 0..size {
        for px in 0..size {
            let (mut body_hits, mut eye_hits) = (0u32, 0u32);
            for sy in 0..N {
                for sx in 0..N {
                    let x = px as f32 + (sx as f32 + 0.5) / N as f32;
                    let y = py as f32 + (sy as f32 + 0.5) / N as f32;
                    if rounded_rect_sdf(x, y, bcx, bcy, bhw, bhh, br) <= 0.0 {
                        body_hits += 1;
                        if eyes.iter().any(|&(ex, ey)| ellipse_inside(x, y, ex, ey, erx, ery)) {
                            eye_hits += 1;
                        }
                    }
                }
            }
            let total = (N * N) as f32;
            let coverage = body_hits as f32 / total;
            if body_hits == 0 {
                continue;
            }
            // Warna rata-rata antara tubuh dan mata pada piksel ini.
            let e = eye_hits as f32 / body_hits as f32;
            let mix = |a: u8, b: u8| (a as f32 * (1.0 - e) + b as f32 * e).round() as u8;
            let i = ((py * size + px) * 4) as usize;
            out[i] = mix(body.r, EYE.r);
            out[i + 1] = mix(body.g, EYE.g);
            out[i + 2] = mix(body.b, EYE.b);
            out[i + 3] = (coverage * 255.0).round() as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(buf: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn ukuran_buffer_benar() {
        assert_eq!(mascot_rgba(32, Skin::Langit).len(), 32 * 32 * 4);
    }

    #[test]
    fn sudut_transparan_tengah_opak() {
        let s = 32;
        let b = mascot_rgba(s, Skin::Mint);
        assert_eq!(px(&b, s, 0, 0)[3], 0);
        assert_eq!(px(&b, s, s - 1, 0)[3], 0);
        let c = px(&b, s, s / 2, (s as f32 * 0.8) as u32);
        assert_eq!(c[3], 255);
        let mint = Skin::Mint.color();
        assert_eq!([c[0], c[1], c[2]], [mint.r, mint.g, mint.b]);
    }

    #[test]
    fn mata_gelap() {
        let s = 64;
        let b = mascot_rgba(s, Skin::Lemon);
        let e = px(&b, s, (s as f32 * 0.36) as u32, s / 2);
        assert!(e[0] < 40 && e[1] < 40 && e[2] < 40, "{e:?}");
    }
}
