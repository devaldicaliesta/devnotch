pub use windows::Foundation::TimeSpan;
pub use windows::UI::Color;
pub use windows::UI::Composition::*;
pub use windows_core::Interface;
pub use windows_numerics::{Vector2, Vector3};

use notchi_core::spring::SpringPreset;
use windows::Foundation::{IReference, PropertyValue, TypedEventHandler};
use windows::System::{DispatcherQueue, DispatcherQueueController};
use windows::UI::Composition::Desktop::DesktopWindowTarget;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::Composition::ICompositorDesktopInterop;
use windows::Win32::System::WinRT::{
    CreateDispatcherQueueController, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT, DispatcherQueueOptions,
};
use windows_core::{HSTRING, Result};

use crate::window::{Event, UiProxy, Window};

pub struct CompositionHost {
    _controller: Option<DispatcherQueueController>,
    pub compositor: Compositor,
    _target: DesktopWindowTarget,
    pub root: ContainerVisual,
}

impl std::fmt::Debug for CompositionHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CompositionHost")
    }
}

impl CompositionHost {
    pub fn new(window: &Window) -> Result<Self> {
        let controller = if DispatcherQueue::GetForCurrentThread().is_ok() {
            None
        } else {
            let options = DispatcherQueueOptions {
                dwSize: size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            };
            Some(unsafe { CreateDispatcherQueueController(options)? })
        };
        let compositor = Compositor::new()?;
        let interop: ICompositorDesktopInterop = compositor.cast()?;
        let target = unsafe { interop.CreateDesktopWindowTarget(HWND(window.raw_hwnd() as *mut _), true)? };
        let root = compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
        target.SetRoot(&root)?;
        Ok(Self { _controller: controller, compositor, _target: target, root })
    }
}

pub fn ts_ms(ms: f32) -> TimeSpan {
    TimeSpan { Duration: (ms.max(0.0) * 10_000.0) as i64 }
}

pub fn color(c: notchi_core::color::Color) -> Color {
    Color { A: c.a, R: c.r, G: c.g, B: c.b }
}

pub fn v2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}

pub fn v3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}

fn boxed_f32(v: f32) -> Result<IReference<f32>> {
    PropertyValue::CreateSingle(v)?.cast()
}

#[derive(Clone)]
pub struct Animator {
    pub compositor: Compositor,
    pub reduce_motion: bool,
}

impl std::fmt::Debug for Animator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animator").field("reduce_motion", &self.reduce_motion).finish()
    }
}

pub const REDUCED_MS: f32 = 120.0;
pub const LOOP_BUDGET_MS: f32 = 12_000.0;

impl Animator {
    pub fn new(compositor: &Compositor) -> Self {
        Self { compositor: compositor.clone(), reduce_motion: false }
    }

    pub fn spring(&self, target: &impl Interface, prop: &str, value: f32, preset: SpringPreset) -> Result<()> {
        let obj: CompositionObject = target.cast()?;
        if self.reduce_motion {
            return self.linear_on(&obj, prop, value, REDUCED_MS, 0.0);
        }
        let anim = self.compositor.CreateSpringScalarAnimation()?;
        anim.SetDampingRatio((1.0 - preset.bounce).clamp(0.05, 1.0))?;
        anim.SetPeriod(ts_ms(preset.duration * 1000.0 / core::f32::consts::TAU))?;
        anim.SetFinalValue(&boxed_f32(value)?)?;
        obj.StartAnimation(&HSTRING::from(prop), &anim)
    }

    pub fn linear(&self, target: &impl Interface, prop: &str, value: f32, ms: f32, delay_ms: f32) -> Result<()> {
        let obj: CompositionObject = target.cast()?;
        let ms = if self.reduce_motion { ms.min(REDUCED_MS) } else { ms };
        self.linear_on(&obj, prop, value, ms, delay_ms)
    }

    pub fn linear_always(&self, target: &impl Interface, prop: &str, value: f32, ms: f32) -> Result<()> {
        let obj: CompositionObject = target.cast()?;
        self.linear_on(&obj, prop, value, ms, 0.0)
    }

    fn linear_on(&self, obj: &CompositionObject, prop: &str, value: f32, ms: f32, delay_ms: f32) -> Result<()> {
        let anim = self.compositor.CreateScalarKeyFrameAnimation()?;
        let linear = self.compositor.CreateLinearEasingFunction()?;
        anim.InsertExpressionKeyFrame(0.0, &HSTRING::from("this.StartingValue"))?;
        anim.InsertKeyFrameWithEasingFunction(1.0, value, &linear)?;
        anim.SetDuration(ts_ms(ms.max(1.0)))?;
        if delay_ms > 0.0 {
            anim.SetDelayTime(ts_ms(delay_ms))?;
            anim.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
        }
        obj.StartAnimation(&HSTRING::from(prop), &anim)
    }

    pub fn repeat(&self, target: &impl Interface, prop: &str, frames: &[(f32, f32)], period_ms: f32, delay_ms: f32) -> Result<()> {
        if self.reduce_motion {
            return Ok(());
        }
        let obj: CompositionObject = target.cast()?;
        let anim = self.keyframes(frames, period_ms)?;
        anim.SetIterationBehavior(AnimationIterationBehavior::Count)?;
        anim.SetIterationCount((LOOP_BUDGET_MS / period_ms.max(1.0)).ceil().max(1.0) as i32)?;
        if delay_ms > 0.0 {
            anim.SetDelayTime(ts_ms(delay_ms))?;
        }
        obj.StartAnimation(&HSTRING::from(prop), &anim)
    }

    pub fn once(&self, target: &impl Interface, prop: &str, frames: &[(f32, f32)], ms: f32, delay_ms: f32) -> Result<()> {
        if self.reduce_motion {
            return Ok(());
        }
        let obj: CompositionObject = target.cast()?;
        let anim = self.keyframes(frames, ms)?;
        if delay_ms > 0.0 {
            anim.SetDelayTime(ts_ms(delay_ms))?;
        }
        obj.StartAnimation(&HSTRING::from(prop), &anim)
    }

    fn keyframes(&self, frames: &[(f32, f32)], ms: f32) -> Result<ScalarKeyFrameAnimation> {
        let anim = self.compositor.CreateScalarKeyFrameAnimation()?;
        let ease = self.compositor.CreateCubicBezierEasingFunction(v2(0.42, 0.0), v2(0.58, 1.0))?;
        for &(p, v) in frames {
            anim.InsertKeyFrameWithEasingFunction(p.clamp(0.0, 1.0), v, &ease)?;
        }
        anim.SetDuration(ts_ms(ms.max(1.0)))?;
        Ok(anim)
    }

    pub fn bind(&self, target: &impl Interface, prop: &str, expression: &str, refs: &[(&str, &CompositionObject)]) -> Result<()> {
        let obj: CompositionObject = target.cast()?;
        let anim = self.compositor.CreateExpressionAnimationWithExpression(&HSTRING::from(expression))?;
        for (name, r) in refs {
            anim.SetReferenceParameter(&HSTRING::from(*name), *r)?;
        }
        obj.StartAnimation(&HSTRING::from(prop), &anim)
    }

    pub fn stop(&self, target: &impl Interface, prop: &str) -> Result<()> {
        let obj: CompositionObject = target.cast()?;
        obj.StopAnimation(&HSTRING::from(prop))
    }

    pub fn batch(&self) -> Result<Batch> {
        Ok(Batch(self.compositor.CreateScopedBatch(CompositionBatchTypes::Animation)?))
    }
}

pub struct Batch(CompositionScopedBatch);

impl std::fmt::Debug for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Batch")
    }
}

impl Batch {
    pub fn end(self, proxy: &UiProxy, tag: u64) -> Result<()> {
        self.0.End()?;
        let proxy = proxy.clone();
        self.0.Completed(&TypedEventHandler::new(move |_, _| {
            proxy.post(Event::User(tag));
            Ok(())
        }))?;
        Ok(())
    }
}
