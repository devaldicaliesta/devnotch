//! Rasterisasi teks, ikon, dan gambar ke permukaan Composition dengan
//! Direct2D/DirectWrite (PRD §6.1, §6.4).
//!
//! Setiap permukaan digambar sekali per perubahan isi, lalu dianimasikan
//! sebagai tekstur oleh compositor. Satu perangkat D3D11 dipakai bersama.
//! Ukuran piksel permukaan = dp × skala DPI, sehingga teks tetap tajam.

use notchi_core::color::Color;
use notchi_core::geom::Size;
use notchi_core::media::Artwork;
use windows::Foundation::Size as WinSize;
use windows::Graphics::DirectX::{DirectXAlphaMode, DirectXPixelFormat};
use windows::UI::Composition::{CompositionDrawingSurface, CompositionGraphicsDevice, Compositor};
use windows::Win32::Foundation::{HMODULE, POINT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_END_CLOSED, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1, D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC, D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
    D2D1CreateFactory, ID2D1Device, ID2D1DeviceContext, ID2D1Factory1,
};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11Device,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FEATURE, DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES, DWRITE_FONT_STRETCH_NORMAL,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_MEDIUM,
    DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_METRICS,
    DWRITE_TEXT_RANGE, DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_WORD_WRAPPING_NO_WRAP,
    DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection, IDWriteTextLayout,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory, WICBitmapDitherTypeNone,
    WICBitmapInterpolationModeHighQualityCubic, WICBitmapPaletteTypeMedianCut, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::WinRT::Composition::{ICompositionDrawingSurfaceInterop, ICompositorInterop};
use windows::core::w;
use windows_core::{HSTRING, Interface, Result};
use windows_numerics::{Matrix3x2, Vector2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl Weight {
    fn dwrite(self) -> DWRITE_FONT_WEIGHT {
        match self {
            Weight::Regular => DWRITE_FONT_WEIGHT_REGULAR,
            Weight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
            Weight::Semibold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
            Weight::Bold => DWRITE_FONT_WEIGHT_BOLD,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub weight: Weight,
    pub color: Color,
    pub max_width: f32,
    /// Angka berlebar sama agar hitung mundur tidak bergoyang.
    pub tabular: bool,
    /// Pakai font ikon sistem (Segoe Fluent Icons).
    pub icon: bool,
    /// Rata tengah di dalam `max_width` (lebar permukaan = `max_width`).
    pub center: bool,
}

impl TextStyle {
    pub fn new(size: f32, weight: Weight, color: Color) -> Self {
        Self { size, weight, color, max_width: 600.0, tabular: false, icon: false, center: false }
    }

    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w.max(1.0);
        self
    }

    pub fn tabular(mut self) -> Self {
        self.tabular = true;
        self
    }

    pub fn icon(mut self) -> Self {
        self.icon = true;
        self
    }

    pub fn centered(mut self) -> Self {
        self.center = true;
        self
    }
}

/// Permukaan siap pakai; `size` dalam dp.
pub struct Surface {
    pub surface: CompositionDrawingSurface,
    pub size: Size,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Surface({}x{})", self.size.w, self.size.h)
    }
}

pub struct Gfx {
    _d3d: ID3D11Device,
    _d2d_device: ID2D1Device,
    d2d_factory: ID2D1Factory1,
    dwrite: IDWriteFactory,
    graphics: CompositionGraphicsDevice,
    text_family: HSTRING,
    icon_family: HSTRING,
}

impl std::fmt::Debug for Gfx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gfx").field("text_family", &self.text_family).finish()
    }
}

fn create_d3d(driver: D3D_DRIVER_TYPE) -> Result<ID3D11Device> {
    let mut device = None;
    // SAFETY: pointer keluaran valid selama pemanggilan.
    unsafe {
        D3D11CreateDevice(
            None,
            driver,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )?;
    }
    device.ok_or_else(|| windows_core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))
}

fn has_family(collection: &IDWriteFontCollection, name: &HSTRING) -> bool {
    let mut index = 0u32;
    let mut exists = windows::core::BOOL(0);
    // SAFETY: nama dan pointer keluaran valid selama pemanggilan.
    unsafe { collection.FindFamilyName(name, &mut index, &mut exists).is_ok() && exists.as_bool() }
}

impl Gfx {
    pub fn new(compositor: &Compositor) -> Result<Self> {
        // GPU perangkat keras bila ada; WARP (perangkat lunak) sebagai cadangan.
        let d3d = create_d3d(D3D_DRIVER_TYPE_HARDWARE).or_else(|_| create_d3d(D3D_DRIVER_TYPE_WARP))?;
        let dxgi: IDXGIDevice = d3d.cast()?;
        // SAFETY: pembuatan factory dan device dengan argumen valid.
        let (d2d_factory, d2d_device, dwrite) = unsafe {
            let factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let device = factory.CreateDevice(&dxgi)?;
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            (factory, device, dwrite)
        };
        let interop: ICompositorInterop = compositor.cast()?;
        // SAFETY: device D2D valid dan hidup selama `Gfx` hidup.
        let graphics = unsafe { interop.CreateGraphicsDevice(&d2d_device)? };

        // Inter (OFL) bila terpasang; Segoe UI Variable bawaan Windows 11
        // sebagai cadangan. SF Pro sengaja tidak dipakai (PRD §2.5).
        let mut collection = None;
        // SAFETY: pointer keluaran valid.
        unsafe { dwrite.GetSystemFontCollection(&mut collection, false)? };
        let pick = |names: &[&str]| -> HSTRING {
            let c = collection.as_ref();
            names
                .iter()
                .map(|n| HSTRING::from(*n))
                .find(|n| c.is_some_and(|c| has_family(c, n)))
                .unwrap_or_else(|| HSTRING::from(*names.last().unwrap_or(&"Segoe UI")))
        };
        let text_family = pick(&["Inter", "Segoe UI Variable Text", "Segoe UI"]);
        let icon_family = pick(&["Segoe Fluent Icons", "Segoe MDL2 Assets"]);
        Ok(Self { _d3d: d3d, _d2d_device: d2d_device, d2d_factory, dwrite, graphics, text_family, icon_family })
    }

    pub fn text_family(&self) -> String {
        self.text_family.to_string()
    }

    fn layout(&self, text: &str, style: &TextStyle) -> Result<(IDWriteTextLayout, Size)> {
        let family = if style.icon { &self.icon_family } else { &self.text_family };
        let utf16: Vec<u16> = text.encode_utf16().collect();
        // SAFETY: semua argumen valid selama pemanggilan.
        unsafe {
            let format = self.dwrite.CreateTextFormat(
                family,
                None,
                style.weight.dwrite(),
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size,
                w!("en-us"),
            )?;
            format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
            let sign = self.dwrite.CreateEllipsisTrimmingSign(&format)?;
            let trimming = DWRITE_TRIMMING {
                granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                delimiter: 0,
                delimiterCount: 0,
            };
            format.SetTrimming(&trimming, &sign)?;
            if style.center {
                format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
            }
            let layout = self.dwrite.CreateTextLayout(&utf16, &format, style.max_width, style.size * 4.0)?;
            if style.tabular {
                let typo = self.dwrite.CreateTypography()?;
                typo.AddFontFeature(DWRITE_FONT_FEATURE { nameTag: DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES, parameter: 1 })?;
                layout.SetTypography(&typo, DWRITE_TEXT_RANGE { startPosition: 0, length: utf16.len() as u32 })?;
            }
            let mut m = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut m)?;
            let w = if style.center { style.max_width } else { m.widthIncludingTrailingWhitespace.min(style.max_width) };
            Ok((layout, Size::new(w.max(1.0), m.height.max(1.0))))
        }
    }

    /// Ukuran teks dalam dp tanpa menggambarnya.
    pub fn measure(&self, text: &str, style: &TextStyle) -> Result<Size> {
        Ok(self.layout(text, style)?.1)
    }

    fn draw(&self, size: Size, scale: f32, paint: impl FnOnce(&ID2D1DeviceContext) -> Result<()>) -> Result<Surface> {
        let pw = (size.w * scale).ceil().max(1.0);
        let ph = (size.h * scale).ceil().max(1.0);
        let surface = self.graphics.CreateDrawingSurface(
            WinSize { Width: pw, Height: ph },
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;
        let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
        let mut offset = POINT::default();
        // SAFETY: BeginDraw/EndDraw berpasangan; konteks hanya dipakai di
        // antara keduanya.
        let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, &mut offset)? };
        // SAFETY: konteks valid sampai EndDraw.
        unsafe {
            dc.SetDpi(96.0 * scale, 96.0 * scale);
            dc.SetTransform(&Matrix3x2::translation(offset.x as f32 / scale, offset.y as f32 / scale));
            dc.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
            dc.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
        let painted = paint(&dc);
        // SAFETY: menutup BeginDraw di atas.
        unsafe { interop.EndDraw()? };
        painted?;
        Ok(Surface { surface, size: Size::new(pw / scale, ph / scale) })
    }

    pub fn text(&self, text: &str, style: &TextStyle, scale: f32) -> Result<Surface> {
        let (layout, size) = self.layout(text, style)?;
        let color = d2d_color(style.color);
        self.draw(size, scale, |dc| {
            // SAFETY: konteks valid selama paint; layout dan brush hidup.
            unsafe {
                let brush = dc.CreateSolidColorBrush(&color, None)?;
                dc.DrawTextLayout(
                    Vector2::new(0.0, 0.0),
                    &layout,
                    &brush,
                    D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                );
            }
            Ok(())
        })
    }

    /// Gambar diisi penuh (crop tengah) ke ukuran `size`.
    pub fn image(&self, art: &Artwork, size: Size, scale: f32) -> Result<Surface> {
        let (iw, ih) = (art.width as f32, art.height as f32);
        // Crop tengah agar rasio `size` terisi tanpa distorsi.
        let target = size.w / size.h.max(1.0);
        let (sw, sh) = if iw / ih > target { (ih * target, ih) } else { (iw, iw / target) };
        let src = D2D_RECT_F { left: (iw - sw) / 2.0, top: (ih - sh) / 2.0, right: (iw + sw) / 2.0, bottom: (ih + sh) / 2.0 };
        let dst = D2D_RECT_F { left: 0.0, top: 0.0, right: size.w, bottom: size.h };
        self.draw(size, scale, |dc| {
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                dpiX: 96.0,
                dpiY: 96.0,
                bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                colorContext: std::mem::ManuallyDrop::new(None),
            };
            // SAFETY: buffer berukuran width × height × 4 (dijamin oleh
            // `decode_artwork`) dan hidup selama pemanggilan.
            unsafe {
                let bmp = dc.CreateBitmap(
                    D2D_SIZE_U { width: art.width, height: art.height },
                    Some(art.bgra.as_ptr() as *const _),
                    art.width * 4,
                    &props,
                )?;
                dc.DrawBitmap(&bmp, Some(&dst), 1.0, D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC, Some(&src), None);
            }
            Ok(())
        })
    }

    /// Ikon kilat untuk mata Notchi saat mengisi daya.
    pub fn bolt(&self, size: Size, color: Color, scale: f32) -> Result<Surface> {
        let pts: [(f32, f32); 7] =
            [(0.62, 0.0), (0.12, 0.58), (0.46, 0.58), (0.36, 1.0), (0.88, 0.40), (0.54, 0.40), (0.66, 0.0)];
        let c = d2d_color(color);
        self.draw(size, scale, |dc| {
            // SAFETY: geometri dan brush dibuat dari factory/konteks yang valid.
            unsafe {
                let geom = self.d2d_factory.CreatePathGeometry()?;
                let sink = geom.Open()?;
                let p = |(x, y): (f32, f32)| Vector2::new(x * size.w, y * size.h);
                sink.BeginFigure(p(pts[0]), D2D1_FIGURE_BEGIN_FILLED);
                let rest: Vec<Vector2> = pts[1..].iter().map(|&q| p(q)).collect();
                sink.AddLines(&rest);
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                sink.Close()?;
                let brush = dc.CreateSolidColorBrush(&c, None)?;
                dc.FillGeometry(&geom, &brush, None);
            }
            Ok(())
        })
    }
}

fn d2d_color(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r: c.r as f32 / 255.0, g: c.g as f32 / 255.0, b: c.b as f32 / 255.0, a: c.a as f32 / 255.0 }
}

/// Batas ukuran berkas sampul yang mau didekode.
pub const MAX_ART_BYTES: usize = 8 * 1024 * 1024;

/// Mendekode sampul album (PNG/JPEG/…) menjadi BGRA premultiplied dengan
/// sisi terpanjang `max_px` (maks. 512 px, PRD §3.6). Dipanggil dari thread
/// yang sudah menginisialisasi COM.
pub fn decode_artwork(bytes: &[u8], max_px: u32) -> Result<Artwork> {
    if bytes.is_empty() || bytes.len() > MAX_ART_BYTES {
        return Err(windows_core::Error::from_hresult(windows::Win32::Foundation::E_INVALIDARG));
    }
    let max_px = max_px.clamp(16, 512);
    // SAFETY: objek WIC dibuat dan dipakai di thread ini; buffer `bytes`
    // hidup sampai decoder selesai (CopyPixels di akhir fungsi).
    unsafe {
        let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let stream = factory.CreateStream()?;
        stream.InitializeFromMemory(bytes)?;
        let decoder = factory.CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)?;
        let frame = decoder.GetFrame(0)?;
        let (mut w, mut h) = (0u32, 0u32);
        frame.GetSize(&mut w, &mut h)?;
        if w == 0 || h == 0 || w > 16_384 || h > 16_384 {
            return Err(windows_core::Error::from_hresult(windows::Win32::Foundation::E_INVALIDARG));
        }
        let ratio = (max_px as f32 / w.max(h) as f32).min(1.0);
        let (nw, nh) = (((w as f32 * ratio).round() as u32).max(1), ((h as f32 * ratio).round() as u32).max(1));
        let scaler = factory.CreateBitmapScaler()?;
        scaler.Initialize(&frame, nw, nh, WICBitmapInterpolationModeHighQualityCubic)?;
        let conv = factory.CreateFormatConverter()?;
        conv.Initialize(&scaler, &GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeMedianCut)?;
        let stride = nw * 4;
        let mut buf = vec![0u8; (stride * nh) as usize];
        conv.CopyPixels(std::ptr::null(), stride, &mut buf)?;
        Ok(Artwork { width: nw, height: nh, bgra: buf })
    }
}
