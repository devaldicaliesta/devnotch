use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Privacy {
    pub mic: bool,
    pub camera: bool,
}

pub const MIC_COLOR: Color = Color::rgb(0xFF9F0A);
pub const CAMERA_COLOR: Color = Color::rgb(0x30D158);

impl Privacy {
    pub fn any(&self) -> bool {
        self.mic || self.camera
    }

    pub fn tint(&self) -> Option<Color> {
        if self.camera {
            Some(CAMERA_COLOR)
        } else if self.mic {
            Some(MIC_COLOR)
        } else {
            None
        }
    }
}

pub fn in_use(last_start: u64, last_stop: u64) -> bool {
    last_start != 0 && last_stop < last_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sedang_dipakai_bila_berhenti_lebih_lama_dari_mulai() {
        assert!(in_use(100, 0), "belum pernah berhenti");
        assert!(in_use(200, 150), "dimulai lagi setelah berhenti");
        assert!(!in_use(100, 100));
        assert!(!in_use(100, 300));
        assert!(!in_use(0, 0), "belum pernah dipakai");
    }

    #[test]
    fn kamera_lebih_diutamakan() {
        assert_eq!(Privacy::default().tint(), None);
        assert_eq!(Privacy { mic: true, camera: false }.tint(), Some(MIC_COLOR));
        assert_eq!(Privacy { mic: true, camera: true }.tint(), Some(CAMERA_COLOR));
        assert!(!Privacy::default().any());
    }
}
