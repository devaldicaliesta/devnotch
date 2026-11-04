//! Melacak perangkat Bluetooth yang tersambung dan terputus.
//!
//! Lapisan platform meneruskan pembaruan mentah dari `DeviceWatcher` (yang hanya membawa
//! properti yang berubah); modul ini menyimpan nama perangkat dan memutuskan kapan alert
//! perlu tampil. Perangkat yang sudah tersambung saat aplikasi mulai tidak memicu alert.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Audio,
    Mouse,
    Keyboard,
    Phone,
    Controller,
    Other,
}

/// Tebakan jenis perangkat dari namanya, untuk memilih ikon.
pub fn kind_of(name: &str) -> DeviceKind {
    let n = name.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| n.contains(w));
    if has(&["bud", "pods", "headphone", "headset", "earphone", "speaker", "wh-", "wf-", "jbl", "soundcore", "sony", "bose", "beats", "audio"]) {
        DeviceKind::Audio
    } else if has(&["mouse", "mx master", "mx anywhere"]) {
        DeviceKind::Mouse
    } else if has(&["keyboard", "keys", "keychron"]) {
        DeviceKind::Keyboard
    } else if has(&["controller", "gamepad", "xbox", "dualsense", "dualshock", "joy-con"]) {
        DeviceKind::Controller
    } else if has(&["phone", "galaxy", "pixel", "redmi", "xiaomi", "oppo", "vivo", "realme"]) {
        DeviceKind::Phone
    } else {
        DeviceKind::Other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Update {
    pub id: String,
    pub name: Option<String>,
    pub connected: Option<bool>,
    pub battery: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub name: String,
    pub kind: DeviceKind,
    pub connected: bool,
    pub battery: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Device {
    id: String,
    name: String,
    connected: bool,
    battery: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Bluetooth {
    devices: Vec<Device>,
    ready: bool,
}

impl Bluetooth {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enumerasi awal selesai; perubahan setelah ini memicu alert.
    pub fn ready(&mut self) {
        self.ready = true;
    }

    pub fn apply(&mut self, u: Update) -> Option<Change> {
        let i = match self.devices.iter().position(|d| d.id == u.id) {
            Some(i) => i,
            None => {
                self.devices.push(Device { id: u.id.clone(), name: String::new(), connected: false, battery: None });
                self.devices.len() - 1
            }
        };
        let d = &mut self.devices[i];
        if let Some(n) = u.name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()) {
            d.name = n;
        }
        if let Some(b) = u.battery {
            d.battery = Some(b.min(100));
        }
        let was = d.connected;
        if let Some(c) = u.connected {
            d.connected = c;
        }
        let flipped = d.connected != was;
        (self.ready && flipped && !d.name.is_empty()).then(|| Change {
            name: d.name.clone(),
            kind: kind_of(&d.name),
            connected: d.connected,
            battery: d.battery.filter(|_| d.connected),
        })
    }

    pub fn remove(&mut self, id: &str) {
        self.devices.retain(|d| d.id != id);
    }

    pub fn connected(&self) -> usize {
        self.devices.iter().filter(|d| d.connected).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn up(id: &str, name: Option<&str>, connected: Option<bool>) -> Update {
        Update { id: id.into(), name: name.map(Into::into), connected, battery: None }
    }

    #[test]
    fn perangkat_awal_tidak_memicu_alert() {
        let mut b = Bluetooth::new();
        assert_eq!(b.apply(up("1", Some("Galaxy Buds2"), Some(true))), None);
        b.ready();
        assert_eq!(b.connected(), 1);
        let c = b.apply(up("1", None, Some(false))).expect("terputus");
        assert_eq!((c.name.as_str(), c.connected, c.kind), ("Galaxy Buds2", false, DeviceKind::Audio));
    }

    #[test]
    fn tersambung_membawa_baterai() {
        let mut b = Bluetooth::new();
        b.apply(up("m", Some("MX Master 3S"), Some(false)));
        b.ready();
        let u = Update { id: "m".into(), name: None, connected: Some(true), battery: Some(140) };
        let c = b.apply(u).unwrap();
        assert_eq!((c.kind, c.battery), (DeviceKind::Mouse, Some(100)));
        assert_eq!(b.apply(up("m", None, Some(true))), None, "tanpa perubahan, tanpa alert");
    }

    #[test]
    fn tanpa_nama_tidak_ada_alert() {
        let mut b = Bluetooth::new();
        b.ready();
        assert_eq!(b.apply(up("x", Some("  "), Some(true))), None);
        b.remove("x");
        assert_eq!(b.connected(), 0);
    }

    #[test]
    fn tebakan_jenis() {
        assert_eq!(kind_of("WH-1000XM5"), DeviceKind::Audio);
        assert_eq!(kind_of("Keychron K2"), DeviceKind::Keyboard);
        assert_eq!(kind_of("Xbox Wireless Controller"), DeviceKind::Controller);
        assert_eq!(kind_of("Tile"), DeviceKind::Other);
    }
}
