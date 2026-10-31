//! Volume perangkat keluaran default (`IAudioEndpointVolume`), untuk scroll
//! di atas pil (PRD §3.3).

use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
use windows_core::Result;

pub struct Volume {
    endpoint: IAudioEndpointVolume,
}

impl std::fmt::Debug for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Volume")
    }
}

impl Volume {
    /// Membuka perangkat keluaran default saat ini.
    pub fn open() -> Result<Volume> {
        // SAFETY: objek COM dibuat di thread ini (sudah diinisialisasi).
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            Ok(Volume { endpoint })
        }
    }

    /// Level 0–1 dan status senyap.
    pub fn get(&self) -> Result<(f32, bool)> {
        // SAFETY: antarmuka valid.
        unsafe { Ok((self.endpoint.GetMasterVolumeLevelScalar()?, self.endpoint.GetMute()?.as_bool())) }
    }

    /// Mengubah level sebesar `delta` (−1–1) dan membatalkan senyap bila naik.
    pub fn step(&self, delta: f32) -> Result<(f32, bool)> {
        let (level, muted) = self.get()?;
        let next = (level + delta).clamp(0.0, 1.0);
        // SAFETY: antarmuka valid; konteks event null (tidak ada GUID).
        unsafe {
            self.endpoint.SetMasterVolumeLevelScalar(next, std::ptr::null())?;
            if muted && delta > 0.0 {
                self.endpoint.SetMute(false, std::ptr::null())?;
            }
        }
        self.get()
    }
}
