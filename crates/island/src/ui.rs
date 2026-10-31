//! Komponen visual kecil di atas Composition: teks, bentuk, pil, maskot.
//!
//! Semua koordinat dalam dp; visual `stage` diskalakan ke piksel fisik.

use notchi_core::color::{Color, Skin};
use notchi_core::geom::{Point, Rect, Size};
use notchi_core::layout::MAX_RADIUS;
use notchi_core::mascot::{Expression, motion_for};
use notchi_core::media::Artwork;
use notchi_platform_win::Result;
use notchi_platform_win::comp::{
    Animator, CompositionObject, CompositionStretch, Compositor, ContainerVisual, Interface, ShapeVisual,
    SpriteVisual, color, v2, v3,
};
use notchi_platform_win::gfx::{Gfx, Surface, TextStyle};

pub const EYE: Color = Color::rgb(0x141414);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Konteks gambar bersama.
#[derive(Debug)]
pub struct Ui {
    pub c: Compositor,
    pub anim: Animator,
    pub gfx: Gfx,
    pub scale: f32,
}

impl Ui {
    pub fn container(&self, parent: &ContainerVisual) -> Result<ContainerVisual> {
        let v = self.c.CreateContainerVisual()?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    fn surface_sprite(&self, parent: &ContainerVisual, s: &Surface, at: Point) -> Result<SpriteVisual> {
        let v = self.c.CreateSpriteVisual()?;
        let brush = self.c.CreateSurfaceBrushWithSurface(&s.surface)?;
        brush.SetStretch(CompositionStretch::Fill)?;
        v.SetBrush(&brush)?;
        v.SetSize(v2(s.size.w, s.size.h))?;
        v.SetOffset(v3(at.x, at.y, 0.0))?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    /// Teks di dalam `rect`, rata vertikal tengah.
    pub fn text(&self, parent: &ContainerVisual, text: &str, style: TextStyle, rect: Rect, align: Align) -> Result<DynText> {
        let style = style.max_width(rect.w);
        let s = self.gfx.text(text, &style, self.scale)?;
        let at = place(rect, s.size, align);
        let visual = self.surface_sprite(parent, &s, at)?;
        Ok(DynText { visual, style, rect, align, last: text.to_string() })
    }

    pub fn image(&self, parent: &ContainerVisual, art: &Artwork, rect: Rect, radius: f32) -> Result<SpriteVisual> {
        let s = self.gfx.image(art, rect.size(), self.scale)?;
        let v = self.surface_sprite(parent, &s, Point::new(rect.x, rect.y))?;
        v.SetSize(v2(rect.w, rect.h))?;
        let geom = self.c.CreateRoundedRectangleGeometry()?;
        geom.SetSize(v2(rect.w, rect.h))?;
        geom.SetCornerRadius(v2(radius, radius))?;
        v.SetClip(&self.c.CreateGeometricClipWithGeometry(&geom)?)?;
        Ok(v)
    }

    pub fn rounded(&self, parent: &ContainerVisual, rect: Rect, radius: f32, fill: Color) -> Result<ShapeVisual> {
        let v = self.c.CreateShapeVisual()?;
        v.SetSize(v2(rect.w, rect.h))?;
        v.SetOffset(v3(rect.x, rect.y, 0.0))?;
        let geom = self.c.CreateRoundedRectangleGeometry()?;
        geom.SetSize(v2(rect.w, rect.h))?;
        geom.SetCornerRadius(v2(radius, radius))?;
        let shape = self.c.CreateSpriteShapeWithGeometry(&geom)?;
        shape.SetFillBrush(&self.c.CreateColorBrushWithColor(color(fill))?)?;
        v.Shapes()?.Append(&shape)?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    /// Batang dengan lebar yang dapat diubah (progres, level volume).
    pub fn bar(&self, parent: &ContainerVisual, rect: Rect, fill: Color) -> Result<SpriteVisual> {
        let v = self.c.CreateSpriteVisual()?;
        v.SetBrush(&self.c.CreateColorBrushWithColor(color(fill))?)?;
        v.SetSize(v2(rect.w, rect.h))?;
        v.SetOffset(v3(rect.x, rect.y, 0.0))?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    /// Tombol bulat dengan glyph atau label di tengah.
    pub fn button(&self, parent: &ContainerVisual, rect: Rect, label: &str, icon: bool, bg: Color, fg: Color) -> Result<ContainerVisual> {
        let host = self.container(parent)?;
        host.SetOffset(v3(rect.x, rect.y, 0.0))?;
        host.SetSize(v2(rect.w, rect.h))?;
        let local = Rect::new(0.0, 0.0, rect.w, rect.h);
        self.rounded(&host, local, rect.h.min(rect.w) / 2.0, bg)?;
        let size = if icon { rect.h * 0.42 } else { 13.0 };
        let mut style = TextStyle::new(size, notchi_platform_win::gfx::Weight::Semibold, fg).centered();
        if icon {
            style = style.icon();
        }
        self.text(&host, label, style, local, Align::Center)?;
        Ok(host)
    }
}

/// Posisi kiri atas untuk isi berukuran `size` di dalam `rect`.
pub fn place(rect: Rect, size: Size, align: Align) -> Point {
    let x = match align {
        Align::Left => rect.x,
        Align::Center => rect.x + (rect.w - size.w) / 2.0,
        Align::Right => rect.x + rect.w - size.w,
    };
    Point::new(x.round(), (rect.y + (rect.h - size.h) / 2.0).round())
}

/// Teks yang isinya diperbarui (hitung mundur, jam, progres).
#[derive(Debug)]
pub struct DynText {
    pub visual: SpriteVisual,
    style: TextStyle,
    rect: Rect,
    align: Align,
    last: String,
}

impl DynText {
    pub fn set(&mut self, ui: &Ui, text: &str) -> Result<()> {
        if text == self.last {
            return Ok(());
        }
        let s = ui.gfx.text(text, &self.style, ui.scale)?;
        let brush = ui.c.CreateSurfaceBrushWithSurface(&s.surface)?;
        brush.SetStretch(CompositionStretch::Fill)?;
        self.visual.SetBrush(&brush)?;
        self.visual.SetSize(v2(s.size.w, s.size.h))?;
        let at = place(self.rect, s.size, self.align);
        self.visual.SetOffset(v3(at.x, at.y, 0.0))?;
        self.last = text.to_string();
        Ok(())
    }
}

/// Pil hitam yang ukurannya dianimasikan pegas; isi dipotong mengikuti
/// bentuknya, dan radius mengikuti busur lingkaran (tinggi / 2).
#[derive(Debug)]
pub struct Pill {
    pub visual: ShapeVisual,
    pub content: ContainerVisual,
}

impl Pill {
    pub fn new(ui: &Ui, parent: &ContainerVisual, fill: Color) -> Result<Pill> {
        let visual = ui.c.CreateShapeVisual()?;
        visual.SetAnchorPoint(v2(0.5, 0.0))?;
        let host: CompositionObject = visual.cast()?;
        let size_expr = "host.Size";
        let radius_expr = format!("Vector2(Min(host.Size.Y / 2, {MAX_RADIUS}), Min(host.Size.Y / 2, {MAX_RADIUS}))");

        let geom = ui.c.CreateRoundedRectangleGeometry()?;
        ui.anim.bind(&geom, "Size", size_expr, &[("host", &host)])?;
        ui.anim.bind(&geom, "CornerRadius", &radius_expr, &[("host", &host)])?;
        let shape = ui.c.CreateSpriteShapeWithGeometry(&geom)?;
        shape.SetFillBrush(&ui.c.CreateColorBrushWithColor(color(fill))?)?;
        visual.Shapes()?.Append(&shape)?;

        let content = ui.c.CreateContainerVisual()?;
        content.SetRelativeSizeAdjustment(v2(1.0, 1.0))?;
        let clip_geom = ui.c.CreateRoundedRectangleGeometry()?;
        ui.anim.bind(&clip_geom, "Size", size_expr, &[("host", &host)])?;
        ui.anim.bind(&clip_geom, "CornerRadius", &radius_expr, &[("host", &host)])?;
        content.SetClip(&ui.c.CreateGeometricClipWithGeometry(&clip_geom)?)?;
        visual.Children()?.InsertAtTop(&content)?;
        parent.Children()?.InsertAtTop(&visual)?;
        Ok(Pill { visual, content })
    }
}

/// Membangun maskot Notchi di `rect` dengan ekspresi dan geraknya.
pub fn mascot(ui: &Ui, parent: &ContainerVisual, rect: Rect, skin: Skin, expr: Expression) -> Result<ContainerVisual> {
    let s = rect.w.min(rect.h);
    let m = motion_for(expr, ui.anim.reduce_motion);
    let host = ui.container(parent)?;
    host.SetOffset(v3(rect.x, rect.y, 0.0))?;
    host.SetSize(v2(s, s))?;
    host.SetCenterPoint(v3(s / 2.0, s * 0.95, 0.0))?;

    // Tubuh: persegi sangat bulat agar terasa lunak, bukan lingkaran sempurna.
    let body = ui.c.CreateShapeVisual()?;
    body.SetSize(v2(s, s))?;
    let geom = ui.c.CreateRoundedRectangleGeometry()?;
    geom.SetOffset(v2(s * 0.04, s * 0.10))?;
    geom.SetSize(v2(s * 0.92, s * 0.84))?;
    geom.SetCornerRadius(v2(s * 0.40, s * 0.40))?;
    let shape = ui.c.CreateSpriteShapeWithGeometry(&geom)?;
    shape.SetFillBrush(&ui.c.CreateColorBrushWithColor(color(skin.color()))?)?;
    body.Shapes()?.Append(&shape)?;

    let shift = m.eye_shift * s;
    let eye_y = s * 0.50;
    let eyes = [(s * 0.36 + shift, eye_y), (s * 0.64 + shift, eye_y)];
    if m.bolt_eyes {
        let size = Size::new(s * 0.20, s * 0.28);
        let bolt = ui.gfx.bolt(size, EYE, ui.scale)?;
        for (cx, cy) in eyes {
            let v = ui.c.CreateSpriteVisual()?;
            let brush = ui.c.CreateSurfaceBrushWithSurface(&bolt.surface)?;
            brush.SetStretch(CompositionStretch::Fill)?;
            v.SetBrush(&brush)?;
            v.SetSize(v2(size.w, size.h))?;
            v.SetOffset(v3(cx - size.w / 2.0, cy - size.h / 2.0, 0.0))?;
            body.Children()?.InsertAtTop(&v)?;
        }
    } else {
        let eye_brush = ui.c.CreateColorBrushWithColor(color(EYE))?;
        for (cx, cy) in eyes {
            let eg = ui.c.CreateEllipseGeometry()?;
            eg.SetCenter(v2(cx, cy))?;
            eg.SetRadius(v2(s * 0.065 * m.eye_scale, s * 0.085 * m.eye_scale))?;
            let eye = ui.c.CreateSpriteShapeWithGeometry(&eg)?;
            eye.SetFillBrush(&eye_brush)?;
            eye.SetCenterPoint(v2(cx, cy))?;
            eye.SetScale(v2(1.0, m.eye_open))?;
            if let Some(period) = m.blink_every {
                let o = m.eye_open;
                ui.anim.repeat(&eye, "Scale.Y", &[(0.0, o), (0.92, o), (0.96, 0.1), (1.0, o)], period as f32, 0.0)?;
            }
            body.Shapes()?.Append(&eye)?;
        }
    }
    host.Children()?.InsertAtTop(&body)?;

    if let Some((lo, hi, period)) = m.breathe {
        let frames = [(0.0, lo), (0.5, hi), (1.0, lo)];
        ui.anim.repeat(&host, "Scale.X", &frames, period as f32, 0.0)?;
        ui.anim.repeat(&host, "Scale.Y", &frames, period as f32, 0.0)?;
    }
    if let Some((deg, period)) = m.sway {
        ui.anim.repeat(&host, "RotationAngleInDegrees", &[(0.0, -deg), (0.5, deg), (1.0, -deg)], period as f32, 0.0)?;
    }
    if m.hop {
        let y = rect.y;
        let h = s * 0.28;
        ui.anim.once(&host, "Offset.Y", &[(0.0, y), (0.35, y - h), (0.7, y), (0.85, y - h * 0.2), (1.0, y)], 700.0, 120.0)?;
    }
    Ok(host)
}

/// Gelombang suara tetap (bukan analisis audio, agar hemat CPU; PRD §2.6).
pub fn waveform(ui: &Ui, parent: &ContainerVisual, rect: Rect, fill: Color, playing: bool) -> Result<()> {
    let bars = 4;
    let bw = 3.0;
    let gap = (rect.w - bw * bars as f32) / (bars as f32 - 1.0);
    let h = rect.h * 0.8;
    let periods = [920.0, 1180.0, 1040.0, 1300.0];
    let peaks = [0.9, 1.0, 0.75, 0.95];
    for i in 0..bars {
        let x = rect.x + i as f32 * (bw + gap);
        let bar = ui.rounded(parent, Rect::new(x, rect.y + (rect.h - h) / 2.0, bw, h), bw / 2.0, fill)?;
        bar.SetCenterPoint(v3(bw / 2.0, h / 2.0, 0.0))?;
        let rest = 0.3;
        bar.SetScale(v3(1.0, rest, 1.0))?;
        if playing {
            let p = peaks[i];
            ui.anim.repeat(&bar, "Scale.Y", &[(0.0, rest), (0.5, p), (1.0, rest)], periods[i], i as f32 * 140.0)?;
        }
    }
    Ok(())
}
