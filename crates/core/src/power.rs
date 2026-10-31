#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerReading {
    pub has_battery: bool,
    pub ac_online: bool,
    pub percent: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    PluggedIn { percent: Option<u8> },
    Unplugged { percent: Option<u8> },
    Low { percent: u8, critical: bool },
}

#[derive(Debug, Clone)]
pub struct BatteryMonitor {
    thresholds: Vec<u8>,
    last: Option<PowerReading>,
    fired_down_to: Option<u8>,
}

impl BatteryMonitor {
    pub fn new(thresholds: &[u8]) -> Self {
        let mut t: Vec<u8> = thresholds.iter().copied().filter(|t| (1..=99).contains(t)).collect();
        t.sort_unstable_by(|a, b| b.cmp(a));
        t.dedup();
        Self { thresholds: t, last: None, fired_down_to: None }
    }

    pub fn last(&self) -> Option<PowerReading> {
        self.last
    }

    pub fn is_low(&self) -> bool {
        match (self.last, self.thresholds.first()) {
            (Some(r), Some(&hi)) => r.has_battery && !r.ac_online && r.percent.is_some_and(|p| p <= hi),
            _ => false,
        }
    }

    pub fn update(&mut self, r: PowerReading) -> Option<PowerEvent> {
        let prev = self.last.replace(r);
        if !r.has_battery {
            return None;
        }
        if r.ac_online {
            self.fired_down_to = None;
        }
        match prev {
            Some(p) if p.has_battery && p.ac_online != r.ac_online => {
                return Some(if r.ac_online {
                    PowerEvent::PluggedIn { percent: r.percent }
                } else {
                    PowerEvent::Unplugged { percent: r.percent }
                });
            }
            _ => {}
        }
        if r.ac_online {
            return None;
        }
        let percent = r.percent?;
        let crossed = self.thresholds.iter().copied().filter(|&t| percent <= t).min()?;
        if self.fired_down_to.is_some_and(|f| f <= crossed) {
            return None;
        }
        self.fired_down_to = Some(crossed);
        let lowest = self.thresholds.last().copied().unwrap_or(crossed);
        Some(PowerEvent::Low { percent, critical: crossed == lowest })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bat(ac: bool, p: u8) -> PowerReading {
        PowerReading { has_battery: true, ac_online: ac, percent: Some(p) }
    }

    #[test]
    fn colok_dan_cabut() {
        let mut m = BatteryMonitor::new(&[20, 10]);
        assert_eq!(m.update(bat(false, 80)), None);
        assert_eq!(m.update(bat(true, 80)), Some(PowerEvent::PluggedIn { percent: Some(80) }));
        assert_eq!(m.update(bat(true, 81)), None);
        assert_eq!(m.update(bat(false, 81)), Some(PowerEvent::Unplugged { percent: Some(81) }));
    }

    #[test]
    fn ambang_sekali_per_siklus() {
        let mut m = BatteryMonitor::new(&[10, 20]);
        m.update(bat(false, 25));
        assert_eq!(m.update(bat(false, 20)), Some(PowerEvent::Low { percent: 20, critical: false }));
        assert_eq!(m.update(bat(false, 19)), None);
        assert!(m.is_low());
        assert_eq!(m.update(bat(false, 10)), Some(PowerEvent::Low { percent: 10, critical: true }));
        assert_eq!(m.update(bat(false, 9)), None);
        m.update(bat(true, 9));
        m.update(bat(false, 9));
        assert_eq!(m.update(bat(false, 8)), Some(PowerEvent::Low { percent: 8, critical: true }));
    }

    #[test]
    fn mulai_dalam_kondisi_rendah() {
        let mut m = BatteryMonitor::new(&[20, 10]);
        assert_eq!(m.update(bat(false, 15)), Some(PowerEvent::Low { percent: 15, critical: false }));
    }

    #[test]
    fn pc_tanpa_baterai_diam() {
        let mut m = BatteryMonitor::new(&[20, 10]);
        let pc = PowerReading { has_battery: false, ac_online: true, percent: None };
        assert_eq!(m.update(pc), None);
        assert_eq!(m.update(PowerReading { ac_online: false, ..pc }), None);
        assert!(!m.is_low());
    }

    #[test]
    fn ambang_tidak_valid_dibuang() {
        let m = BatteryMonitor::new(&[0, 100, 20, 20]);
        assert_eq!(m.thresholds, vec![20]);
    }
}
