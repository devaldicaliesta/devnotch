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
    pub fn open() -> Result<Volume> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            Ok(Volume { endpoint })
        }
    }

    pub fn get(&self) -> Result<(f32, bool)> {
        unsafe { Ok((self.endpoint.GetMasterVolumeLevelScalar()?, self.endpoint.GetMute()?.as_bool())) }
    }

    pub fn step(&self, delta: f32) -> Result<(f32, bool)> {
        let (level, muted) = self.get()?;
        let next = (level + delta).clamp(0.0, 1.0);
        unsafe {
            self.endpoint.SetMasterVolumeLevelScalar(next, std::ptr::null())?;
            if muted && delta > 0.0 {
                self.endpoint.SetMute(false, std::ptr::null())?;
            }
        }
        self.get()
    }
}
