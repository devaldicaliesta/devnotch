use std::rc::Rc;

use notchi_core::color::{Color, Skin, TEXT_PRIMARY};
use notchi_core::geom::{Point, Rect, Size};
use notchi_core::layout::{FILLET, MAX_RADIUS};
use notchi_core::mascot::{Expression, motion_for};
use notchi_core::media::Artwork;
use notchi_core::spring::SpringPreset;
use notchi_core::theme::{Palette, Relief, Style};
use notchi_platform_win::Result;
use notchi_platform_win::comp::{
    Animator, CompositionObject, CompositionSpriteShape, CompositionStretch, Compositor, ContainerVisual, Interface,
    ShapeVisual, SpriteVisual, color, v2, v3,
};
use notchi_platform_win::gfx::{Gfx, Surface, TextStyle, Weight};

pub const EYE: Color = Color::rgb(0x141414);
const BLUSH: Color = Color::rgb(0xFF6F91);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tone {
    Neutral,
    Tinted(Color),
    Disabled,
}

#[derive(Debug)]
pub struct Ui {
    pub c: Compositor,
    pub anim: Animator,
    pub gfx: Rc<Gfx>,
    pub scale: f32,
    pub pal: Palette,
    pub accent: Color,
}

impl Ui {
    pub fn fork(&self, scale: f32) -> Ui {
        Ui { c: self.c.clone(), anim: self.anim.clone(), gfx: self.gfx.clone(), scale, pal: self.pal, accent: self.accent }
    }

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

    pub fn ring(&self, parent: &ContainerVisual, rect: Rect, stroke: f32, c: Color) -> Result<ShapeVisual> {
        let v = self.c.CreateShapeVisual()?;
        v.SetSize(v2(rect.w, rect.h))?;
        v.SetOffset(v3(rect.x, rect.y, 0.0))?;
        let geom = self.c.CreateEllipseGeometry()?;
        geom.SetCenter(v2(rect.w / 2.0, rect.h / 2.0))?;
        geom.SetRadius(v2(rect.w / 2.0 - stroke / 2.0, rect.h / 2.0 - stroke / 2.0))?;
        let shape = self.c.CreateSpriteShapeWithGeometry(&geom)?;
        shape.SetStrokeBrush(&self.c.CreateColorBrushWithColor(color(c))?)?;
        shape.SetStrokeThickness(stroke)?;
        v.Shapes()?.Append(&shape)?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    pub fn bar(&self, parent: &ContainerVisual, rect: Rect, fill: Color) -> Result<SpriteVisual> {
        let v = self.c.CreateSpriteVisual()?;
        v.SetBrush(&self.c.CreateColorBrushWithColor(color(fill))?)?;
        v.SetSize(v2(rect.w, rect.h))?;
        v.SetOffset(v3(rect.x, rect.y, 0.0))?;
        parent.Children()?.InsertAtTop(&v)?;
        Ok(v)
    }

    pub fn relief(&self, parent: &ContainerVisual, rect: Rect, radius: f32, r: &Relief) -> Result<SpriteVisual> {
        let (s, pad) = self.gfx.relief(rect.size(), radius, r, self.scale)?;
        self.surface_sprite(parent, &s, Point::new(rect.x - pad, rect.y - pad))
    }

    pub fn surface(&self, parent: &ContainerVisual, rect: Rect, radius: f32, raised: bool, fill: Color) -> Result<()> {
        let depth = (rect.h.min(rect.w) * 0.09).clamp(2.0, 6.0);
        let r = if raised { self.pal.raised(fill, depth) } else { self.pal.inset(fill, depth * 0.7) };
        if self.pal.is_flat() {
            self.rounded(parent, rect, radius, fill)?;
        } else {
            self.relief(parent, rect, radius, &r)?;
        }
        Ok(())
    }

    pub fn tone_colors(&self, tone: Tone) -> (Color, Color) {
        let p = &self.pal;
        match (p.style, tone) {
            (_, Tone::Disabled) => (p.well, TEXT_PRIMARY.with_opacity(0.3)),
            (Style::Classic, Tone::Neutral) => (TEXT_PRIMARY.with_opacity(0.14).over(p.base), TEXT_PRIMARY),
            (Style::Classic, Tone::Tinted(c)) => (c.with_opacity(0.25).over(p.base), c),
            (Style::Neumorphism, Tone::Neutral) => (p.base, TEXT_PRIMARY),
            (Style::Neumorphism, Tone::Tinted(c)) => (p.base, c),
            (Style::Clay, Tone::Neutral) => (Color::rgb(0x2A2833), TEXT_PRIMARY),
            (Style::Clay, Tone::Tinted(c)) => (c, p.on(c)),
        }
    }

    pub fn button(&self, parent: &ContainerVisual, rect: Rect, label: &str, icon: bool, tone: Tone) -> Result<ContainerVisual> {
        let host = self.container(parent)?;
        host.SetOffset(v3(rect.x, rect.y, 0.0))?;
        host.SetSize(v2(rect.w, rect.h))?;
        host.SetCenterPoint(v3(rect.w / 2.0, rect.h / 2.0, 0.0))?;
        let local = Rect::new(0.0, 0.0, rect.w, rect.h);
        let (bg, fg) = self.tone_colors(tone);
        let radius = rect.h.min(rect.w) / 2.0;
        match (self.pal.style, tone) {
            (Style::Classic, _) => {
                self.rounded(&host, local, radius, bg)?;
            }
            (_, Tone::Disabled) => {
                let depth = (rect.h * 0.07).clamp(1.5, 4.0);
                self.relief(&host, local, radius, &self.pal.inset(bg, depth))?;
            }
            (Style::Clay, _) => {
                let depth = (rect.h * 0.1).clamp(2.0, 5.0);
                self.relief(&host, local, radius, &self.pal.clay(bg, depth))?;
            }
            _ => {
                let depth = (rect.h * 0.1).clamp(2.0, 5.0);
                self.relief(&host, local, radius, &self.pal.raised(bg, depth))?;
            }
        }
        let size = if icon { rect.h * 0.42 } else { 13.0 };
        let mut style = TextStyle::new(size, Weight::Semibold, fg).centered();
        if icon {
            style = style.icon();
        }
        self.text(&host, label, style, local, Align::Center)?;
        Ok(host)
    }

    pub fn pop(&self, v: &ContainerVisual, scale: f32) -> Result<()> {
        self.anim.spring(v, "Scale.X", scale, SpringPreset::SNAPPY)?;
        self.anim.spring(v, "Scale.Y", scale, SpringPreset::SNAPPY)
    }
}

pub fn place(rect: Rect, size: Size, align: Align) -> Point {
    let x = match align {
        Align::Left => rect.x,
        Align::Center => rect.x + (rect.w - size.w) / 2.0,
        Align::Right => rect.x + rect.w - size.w,
    };
    Point::new(x.round(), (rect.y + (rect.h - size.h) / 2.0).round())
}

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

#[derive(Debug)]
pub struct Pill {
    pub visual: ShapeVisual,
    pub content: ContainerVisual,
    flares: [SpriteVisual; 2],
    shape: CompositionSpriteShape,
    fill: Color,
}

const OVERLAP: f32 = 1.0;

impl Pill {
    pub fn new(ui: &Ui, parent: &ContainerVisual, fill: Color) -> Result<Pill> {
        let flares = [ui.c.CreateSpriteVisual()?, ui.c.CreateSpriteVisual()?];
        for f in &flares {
            parent.Children()?.InsertAtTop(f)?;
        }
        let visual = ui.c.CreateShapeVisual()?;
        visual.SetAnchorPoint(v2(0.5, 0.0))?;
        let host: CompositionObject = visual.cast()?;
        let refs = [("host", &host)];

        let r = format!("Min(Min(host.Size.Y / 2, host.Size.X / 2), {MAX_RADIUS})");
        let offset = format!("Vector2(0, -{r})");
        let size = format!("Vector2(host.Size.X, host.Size.Y + {r})");
        let radius = format!("Vector2({r}, {r})");
        let attached = |ui: &Ui| -> Result<_> {
            let g = ui.c.CreateRoundedRectangleGeometry()?;
            ui.anim.bind(&g, "Offset", &offset, &refs)?;
            ui.anim.bind(&g, "Size", &size, &refs)?;
            ui.anim.bind(&g, "CornerRadius", &radius, &refs)?;
            Ok(g)
        };

        let shape = ui.c.CreateSpriteShapeWithGeometry(&attached(ui)?)?;
        shape.SetFillBrush(&ui.c.CreateColorBrushWithColor(color(fill))?)?;
        visual.Shapes()?.Append(&shape)?;

        let content = ui.c.CreateContainerVisual()?;
        content.SetRelativeSizeAdjustment(v2(1.0, 1.0))?;
        content.SetClip(&ui.c.CreateGeometricClipWithGeometry(&attached(ui)?)?)?;
        visual.Children()?.InsertAtTop(&content)?;
        parent.Children()?.InsertAtTop(&visual)?;

        let f = FILLET;
        let grow = format!("Clamp(host.Size.Y / {}, 0, 1)", 2.0 * f);
        let edge = "host.Size.X * host.Scale.X / 2";
        let sides = [(format!("host.Offset.X - {edge} - {f}"), f), (format!("host.Offset.X + {edge} - {OVERLAP}"), OVERLAP)];
        for (flare, (x, pivot)) in flares.iter().zip(sides) {
            flare.SetSize(v2(f + OVERLAP, f))?;
            flare.SetCenterPoint(v3(pivot, 0.0, 0.0))?;
            ui.anim.bind(flare, "Offset", &format!("Vector3({x}, host.Offset.Y, 0)"), &refs)?;
            ui.anim.bind(flare, "Scale", &format!("Vector3({grow}, {grow}, 1)"), &refs)?;
            ui.anim.bind(flare, "Opacity", "host.Opacity", &refs)?;
        }
        Ok(Pill { visual, content, flares, shape, fill })
    }

    pub fn set_fill(&mut self, ui: &Ui, fill: Color) -> Result<()> {
        self.fill = fill;
        self.shape.SetFillBrush(&ui.c.CreateColorBrushWithColor(color(fill))?)
    }

    pub fn set_liquid(&self, ui: &Ui, on: bool) -> Result<()> {
        for (i, flare) in self.flares.iter().enumerate() {
            flare.SetIsVisible(on)?;
            if on {
                let s = ui.gfx.fillet(FILLET, OVERLAP, self.fill, ui.scale, i == 1)?;
                let brush = ui.c.CreateSurfaceBrushWithSurface(&s.surface)?;
                brush.SetStretch(CompositionStretch::Fill)?;
                flare.SetBrush(&brush)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Mascot {
    pub rect: Rect,
    host: ContainerVisual,
    pose: ContainerVisual,
    eyes: ContainerVisual,
    blush: ShapeVisual,
    size: f32,
    look: (i8, i8),
    perked: bool,
}

impl Mascot {
    pub fn center(&self) -> Point {
        self.rect.center()
    }

    pub fn look(&mut self, ui: &Ui, dx: f32, dy: f32) -> Result<()> {
        let q = ((dx.clamp(-1.0, 1.0) * 5.0).round() as i8, (dy.clamp(-1.0, 1.0) * 5.0).round() as i8);
        if q == self.look {
            return Ok(());
        }
        self.look = q;
        let s = self.size;
        ui.anim.spring(&self.eyes, "Offset.X", q.0 as f32 / 5.0 * s * 0.08, SpringPreset::SNAPPY)?;
        ui.anim.spring(&self.eyes, "Offset.Y", q.1 as f32 / 5.0 * s * 0.06, SpringPreset::SNAPPY)
    }

    pub fn perk(&mut self, ui: &Ui, on: bool) -> Result<()> {
        if on == self.perked {
            return Ok(());
        }
        self.perked = on;
        ui.pop(&self.pose, if on { 1.1 } else { 1.0 })
    }

    pub fn boop(&mut self, ui: &Ui) -> Result<()> {
        let rest = if self.perked { 1.1 } else { 1.0 };
        let sx = [(0.0, rest), (0.16, rest * 1.2), (0.38, rest * 0.88), (0.62, rest * 1.06), (1.0, rest)];
        let sy = [(0.0, rest), (0.16, rest * 0.8), (0.38, rest * 1.12), (0.62, rest * 0.96), (1.0, rest)];
        ui.anim.once(&self.pose, "Scale.X", &sx, 560.0, 0.0)?;
        ui.anim.once(&self.pose, "Scale.Y", &sy, 560.0, 0.0)?;
        ui.anim.once(&self.eyes, "Scale.Y", &[(0.0, 1.0), (0.12, 0.2), (0.78, 0.2), (1.0, 1.0)], 950.0, 0.0)?;
        ui.anim.once(&self.blush, "Opacity", &[(0.0, 0.0), (0.15, 1.0), (0.8, 1.0), (1.0, 0.0)], 1_200.0, 0.0)?;
        let y = self.rect.y;
        let h = self.size * 0.22;
        ui.anim.once(&self.host, "Offset.Y", &[(0.0, y), (0.3, y - h), (0.62, y), (0.8, y - h * 0.25), (1.0, y)], 620.0, 60.0)
    }
}

pub fn mascot(ui: &Ui, parent: &ContainerVisual, rect: Rect, skin: Skin, expr: Expression) -> Result<Mascot> {
    let s = rect.w.min(rect.h);
    let rect = Rect::new(rect.x, rect.y, s, s);
    let m = motion_for(expr, ui.anim.reduce_motion);
    let host = ui.container(parent)?;
    host.SetOffset(v3(rect.x, rect.y, 0.0))?;
    host.SetSize(v2(s, s))?;
    let pose = ui.container(&host)?;
    pose.SetSize(v2(s, s))?;
    pose.SetCenterPoint(v3(s / 2.0, s * 0.95, 0.0))?;
    let body = ui.container(&pose)?;
    body.SetSize(v2(s, s))?;
    body.SetCenterPoint(v3(s / 2.0, s * 0.95, 0.0))?;

    let shell = Rect::new(s * 0.04, s * 0.10, s * 0.92, s * 0.84);
    if ui.pal.is_flat() {
        ui.rounded(&body, shell, s * 0.40, skin.color())?;
    } else {
        let depth = (s * 0.06).clamp(1.2, 3.5);
        ui.relief(&body, shell, s * 0.40, &ui.pal.clay(skin.color(), depth))?;
    }

    let blush = ui.c.CreateShapeVisual()?;
    blush.SetSize(v2(s, s))?;
    blush.SetOpacity(0.0)?;
    let blush_brush = ui.c.CreateColorBrushWithColor(color(BLUSH.with_opacity(0.55)))?;
    for cx in [s * 0.24, s * 0.76] {
        let g = ui.c.CreateEllipseGeometry()?;
        g.SetCenter(v2(cx, s * 0.66))?;
        g.SetRadius(v2(s * 0.08, s * 0.05))?;
        let b = ui.c.CreateSpriteShapeWithGeometry(&g)?;
        b.SetFillBrush(&blush_brush)?;
        blush.Shapes()?.Append(&b)?;
    }
    body.Children()?.InsertAtTop(&blush)?;

    let eyes = ui.container(&body)?;
    eyes.SetSize(v2(s, s))?;
    eyes.SetCenterPoint(v3(s / 2.0, s * 0.5, 0.0))?;
    let shift = m.eye_shift * s;
    let eye_y = s * 0.50;
    let centers = [(s * 0.36 + shift, eye_y), (s * 0.64 + shift, eye_y)];
    if m.bolt_eyes {
        let size = Size::new(s * 0.20, s * 0.28);
        let bolt = ui.gfx.bolt(size, EYE, ui.scale)?;
        for (cx, cy) in centers {
            let v = ui.c.CreateSpriteVisual()?;
            let brush = ui.c.CreateSurfaceBrushWithSurface(&bolt.surface)?;
            brush.SetStretch(CompositionStretch::Fill)?;
            v.SetBrush(&brush)?;
            v.SetSize(v2(size.w, size.h))?;
            v.SetOffset(v3(cx - size.w / 2.0, cy - size.h / 2.0, 0.0))?;
            eyes.Children()?.InsertAtTop(&v)?;
        }
    } else {
        let layer = ui.c.CreateShapeVisual()?;
        layer.SetSize(v2(s, s))?;
        let eye_brush = ui.c.CreateColorBrushWithColor(color(EYE))?;
        let shine = ui.c.CreateColorBrushWithColor(color(TEXT_PRIMARY.with_opacity(0.85)))?;
        for (cx, cy) in centers {
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
            layer.Shapes()?.Append(&eye)?;
            if !ui.pal.is_flat() && s >= 30.0 && m.eye_open > 0.5 {
                let hg = ui.c.CreateEllipseGeometry()?;
                hg.SetCenter(v2(cx + s * 0.022, cy - s * 0.03))?;
                hg.SetRadius(v2(s * 0.02, s * 0.02))?;
                let h = ui.c.CreateSpriteShapeWithGeometry(&hg)?;
                h.SetFillBrush(&shine)?;
                layer.Shapes()?.Append(&h)?;
            }
        }
        eyes.Children()?.InsertAtTop(&layer)?;
    }

    if let Some((lo, hi, period)) = m.breathe {
        let frames = [(0.0, lo), (0.5, hi), (1.0, lo)];
        ui.anim.repeat(&body, "Scale.X", &frames, period as f32, 0.0)?;
        ui.anim.repeat(&body, "Scale.Y", &frames, period as f32, 0.0)?;
    }
    if let Some((deg, period)) = m.sway {
        ui.anim.repeat(&body, "RotationAngleInDegrees", &[(0.0, 0.0), (0.25, deg), (0.75, -deg), (1.0, 0.0)], period as f32, 0.0)?;
    }
    if m.hop {
        let y = rect.y;
        let h = s * 0.28;
        ui.anim.once(&host, "Offset.Y", &[(0.0, y), (0.35, y - h), (0.7, y), (0.85, y - h * 0.2), (1.0, y)], 700.0, 120.0)?;
    }
    Ok(Mascot { rect, host, pose, eyes, blush, size: s, look: (0, 0), perked: false })
}

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
