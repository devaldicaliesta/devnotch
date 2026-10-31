use notchi_core::color::{Color, TEXT_PRIMARY};
use notchi_core::geom::{Point, Rect};
use notchi_platform_win::Result;
use notchi_platform_win::comp::{CompositionStretch, ContainerVisual, Interface, Visual, v2, v3};
use notchi_platform_win::window::UiProxy;

use crate::ui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Burst {
    Sparkle,
    Confetti,
    Pop,
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }
}

const EASE_OUT: [f32; 4] = [0.15, 0.85, 0.35, 1.0];
const GRAVITY: [f32; 4] = [0.45, 0.0, 0.75, 1.0];
const SOFT: [f32; 4] = [0.4, 0.0, 0.6, 1.0];

#[derive(Debug, Clone, Copy)]
pub struct Spark<'a> {
    pub at: Point,
    pub kind: Burst,
    pub colors: &'a [Color],
    pub seed: u64,
}

pub fn burst(ui: &Ui, parent: &ContainerVisual, spark: Spark<'_>, proxy: &UiProxy, tag: u64) -> Result<Option<ContainerVisual>> {
    let Spark { at, kind, colors, seed } = spark;
    if ui.anim.reduce_motion || colors.is_empty() {
        return Ok(None);
    }
    let mut rng = Rng(seed | 1);
    let root = ui.container(parent)?;
    let batch = ui.anim.batch()?;
    let (count, life) = match kind {
        Burst::Sparkle => (12, 720.0),
        Burst::Confetti => (20, 1_050.0),
        Burst::Pop => (8, 480.0),
    };
    let glows: Vec<_> = colors
        .iter()
        .take(4)
        .map(|c| ui.gfx.glow(16.0, *c, ui.scale))
        .collect::<Result<_>>()?;
    for i in 0..count {
        let ci = i % colors.len().min(4);
        let angle = (i as f32 / count as f32) * core::f32::consts::TAU + rng.range(-0.3, 0.3);
        let (dist, size) = match kind {
            Burst::Sparkle => (rng.range(18.0, 34.0), rng.range(5.0, 9.0)),
            Burst::Confetti => (rng.range(26.0, 54.0), rng.range(4.0, 7.0)),
            Burst::Pop => (rng.range(10.0, 18.0), rng.range(4.0, 6.0)),
        };
        let dx = angle.cos() * dist;
        let dy = match kind {
            Burst::Sparkle | Burst::Pop => angle.sin() * dist,
            Burst::Confetti => -angle.sin().abs().max(0.25) * dist,
        };
        let fall = match kind {
            Burst::Confetti => rng.range(28.0, 46.0),
            Burst::Sparkle => rng.range(4.0, 10.0),
            Burst::Pop => 0.0,
        };
        let x0 = at.x - size / 2.0;
        let y0 = at.y - size / 2.0;
        let v = if kind == Burst::Confetti {
            let r = ui.rounded(&root, Rect::new(0.0, 0.0, size * 0.6, size * 1.3), size * 0.2, colors[ci])?;
            r.SetCenterPoint(v3(size * 0.3, size * 0.65, 0.0))?;
            let spin = rng.range(-540.0, 540.0);
            ui.anim.curve(&r, "RotationAngleInDegrees", &[(0.0, 0.0), (1.0, spin)], life, 0.0, SOFT)?;
            r.cast::<Visual>()?
        } else {
            let s = ui.c.CreateSpriteVisual()?;
            let brush = ui.c.CreateSurfaceBrushWithSurface(&glows[ci].surface)?;
            brush.SetStretch(CompositionStretch::Fill)?;
            s.SetBrush(&brush)?;
            s.SetSize(v2(size, size))?;
            s.SetCenterPoint(v3(size / 2.0, size / 2.0, 0.0))?;
            root.Children()?.InsertAtTop(&s)?;
            s.cast::<Visual>()?
        };
        v.SetOffset(v3(x0, y0, 0.0))?;
        let delay = rng.range(0.0, 60.0);
        let life = life * rng.range(0.8, 1.15);
        ui.anim.curve(&v, "Offset.X", &[(0.0, x0), (1.0, x0 + dx)], life, delay, EASE_OUT)?;
        ui.anim.curve(&v, "Offset.Y", &[(0.0, y0), (0.45, y0 + dy), (1.0, y0 + dy + fall)], life, delay, GRAVITY)?;
        let peak = rng.range(0.9, 1.3);
        ui.anim.curve(&v, "Scale.X", &[(0.0, 0.3), (0.2, peak), (1.0, 0.2)], life, delay, SOFT)?;
        ui.anim.curve(&v, "Scale.Y", &[(0.0, 0.3), (0.2, peak), (1.0, 0.2)], life, delay, SOFT)?;
        ui.anim.curve(&v, "Opacity", &[(0.0, 1.0), (0.65, 1.0), (1.0, 0.0)], life, delay, SOFT)?;
    }
    batch.end(proxy, tag)?;
    Ok(Some(root))
}

pub fn sparkle_colors(accent: Color) -> [Color; 3] {
    [accent, TEXT_PRIMARY, accent.with_opacity(0.7)]
}
