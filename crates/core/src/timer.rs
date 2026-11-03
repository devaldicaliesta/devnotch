use crate::Ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountdownState {
    Running { end: Ms },
    Paused { remaining: Ms },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Countdown {
    total: Ms,
    state: CountdownState,
}

pub const MAX_COUNTDOWN: Ms = 24 * 60 * 60 * 1000;

impl Countdown {
    pub fn start(duration: Ms, now: Ms) -> Self {
        let duration = duration.clamp(1_000, MAX_COUNTDOWN);
        Self { total: duration, state: CountdownState::Running { end: now + duration } }
    }

    pub fn total(&self) -> Ms {
        self.total
    }

    pub fn remaining(&self, now: Ms) -> Ms {
        match self.state {
            CountdownState::Running { end } => end.saturating_sub(now),
            CountdownState::Paused { remaining } => remaining,
        }
    }

    pub fn progress(&self, now: Ms) -> f32 {
        1.0 - self.remaining(now) as f32 / self.total.max(1) as f32
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, CountdownState::Running { .. })
    }

    pub fn is_finished(&self, now: Ms) -> bool {
        self.remaining(now) == 0
    }

    pub fn pause(&mut self, now: Ms) {
        if let CountdownState::Running { end } = self.state {
            self.state = CountdownState::Paused { remaining: end.saturating_sub(now) };
        }
    }

    pub fn resume(&mut self, now: Ms) {
        if let CountdownState::Paused { remaining } = self.state {
            self.state = CountdownState::Running { end: now + remaining };
        }
    }

    pub fn toggle(&mut self, now: Ms) {
        if self.is_running() { self.pause(now) } else { self.resume(now) }
    }

    pub fn add(&mut self, extra: Ms, now: Ms) {
        let remaining = (self.remaining(now) + extra).min(MAX_COUNTDOWN);
        self.total = self.total.max(remaining);
        self.state = match self.state {
            CountdownState::Running { .. } => CountdownState::Running { end: now + remaining },
            CountdownState::Paused { .. } => CountdownState::Paused { remaining },
        };
    }

    pub fn subtract(&mut self, less: Ms, now: Ms) -> bool {
        let remaining = self.remaining(now);
        if remaining <= less {
            return false;
        }
        let remaining = remaining - less;
        self.state = match self.state {
            CountdownState::Running { .. } => CountdownState::Running { end: now + remaining },
            CountdownState::Paused { .. } => CountdownState::Paused { remaining },
        };
        true
    }

    pub fn next_tick(&self, now: Ms) -> Option<Ms> {
        match self.state {
            CountdownState::Running { end } if end > now => {
                let rem = end - now;
                let to_boundary = match rem % 1000 {
                    0 => 1000,
                    r => r,
                };
                Some(now + to_boundary.min(rem))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stopwatch {
    accumulated: Ms,
    started_at: Option<Ms>,
}

impl Stopwatch {
    pub fn start(now: Ms) -> Self {
        Self { accumulated: 0, started_at: Some(now) }
    }

    pub fn elapsed(&self, now: Ms) -> Ms {
        self.accumulated + self.started_at.map_or(0, |s| now.saturating_sub(s))
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    pub fn toggle(&mut self, now: Ms) {
        match self.started_at.take() {
            Some(s) => self.accumulated += now.saturating_sub(s),
            None => self.started_at = Some(now),
        }
    }

    pub fn reset(&mut self, now: Ms) {
        self.accumulated = 0;
        if self.started_at.is_some() {
            self.started_at = Some(now);
        }
    }

    pub fn next_tick(&self, now: Ms) -> Option<Ms> {
        self.is_running().then(|| now + (1000 - self.elapsed(now) % 1000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hitung_mundur_dasar() {
        let t = Countdown::start(5_000, 1_000);
        assert_eq!(t.remaining(1_000), 5_000);
        assert_eq!(t.remaining(3_500), 2_500);
        assert!(!t.is_finished(5_999));
        assert!(t.is_finished(6_000));
        assert_eq!(t.remaining(99_000), 0);
    }

    #[test]
    fn jeda_dan_lanjut() {
        let mut t = Countdown::start(10_000, 0);
        t.pause(4_000);
        assert_eq!(t.remaining(50_000), 6_000);
        assert_eq!(t.next_tick(50_000), None);
        t.resume(50_000);
        assert_eq!(t.remaining(51_000), 5_000);
        t.toggle(52_000);
        assert!(!t.is_running());
    }

    #[test]
    fn tambah_satu_menit() {
        let mut t = Countdown::start(30_000, 0);
        t.add(60_000, 10_000);
        assert_eq!(t.remaining(10_000), 80_000);
        assert!(t.total() >= 80_000);
        assert!(t.progress(10_000) >= 0.0);
    }

    #[test]
    fn kurangi_lewat_scroll_tidak_menamatkan_timer() {
        let mut t = Countdown::start(150_000, 0);
        assert!(t.subtract(60_000, 10_000));
        assert_eq!(t.remaining(10_000), 80_000);
        assert!(t.is_running());
        assert!(!t.subtract(60_000, 30_000), "sisa 60 detik: tidak dikurangi sampai habis");
        assert_eq!(t.remaining(30_000), 60_000);
        t.pause(40_000);
        assert!(!t.subtract(60_000, 40_000));
        let mut p = Countdown::start(300_000, 0);
        p.pause(0);
        assert!(p.subtract(60_000, 99_000));
        assert_eq!(p.remaining(200_000), 240_000, "timer dijeda tetap dijeda");
        assert!(!p.is_running());
    }

    #[test]
    fn batas_durasi() {
        assert_eq!(Countdown::start(0, 0).total(), 1_000);
        assert_eq!(Countdown::start(Ms::MAX / 2, 0).total(), MAX_COUNTDOWN);
        let mut t = Countdown::start(MAX_COUNTDOWN, 0);
        t.add(MAX_COUNTDOWN, 0);
        assert_eq!(t.remaining(0), MAX_COUNTDOWN);
    }

    #[test]
    fn tick_tepat_di_batas_detik() {
        let t = Countdown::start(5_000, 0);
        assert_eq!(t.next_tick(0), Some(1_000));
        assert_eq!(t.next_tick(250), Some(1_000));
        assert_eq!(t.next_tick(4_900), Some(5_000));
        assert_eq!(t.next_tick(5_000), None);
    }

    #[test]
    fn stopwatch() {
        let mut s = Stopwatch::start(1_000);
        assert_eq!(s.elapsed(3_500), 2_500);
        assert_eq!(s.next_tick(3_500), Some(4_000));
        s.toggle(4_000);
        assert_eq!(s.elapsed(10_000), 3_000);
        assert_eq!(s.next_tick(10_000), None);
        s.toggle(10_000);
        assert_eq!(s.elapsed(11_000), 4_000);
        s.reset(11_000);
        assert_eq!(s.elapsed(11_500), 500);
    }
}
