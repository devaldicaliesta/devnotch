use crate::Ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackStatus {
    #[default]
    Closed,
    Opened,
    Changing,
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Controls {
    pub play: bool,
    pub pause: bool,
    pub toggle: bool,
    pub next: bool,
    pub prev: bool,
    pub seek: bool,
}

impl Controls {
    pub fn can_play_pause(&self, playing: bool) -> bool {
        self.toggle || if playing { self.pause } else { self.play }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeline {
    pub position: Ms,
    pub end: Ms,
    pub reported_at: Ms,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Artwork {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl core::fmt::Debug for Artwork {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Artwork({}x{})", self.width, self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MediaSnapshot {
    pub source_app_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub status: PlaybackStatus,
    pub controls: Controls,
    pub timeline: Option<Timeline>,
    pub session_index: usize,
    pub session_count: usize,
    pub track_id: u64,
}

impl MediaSnapshot {
    pub fn is_playing(&self) -> bool {
        self.status == PlaybackStatus::Playing
    }

    pub fn has_content(&self) -> bool {
        !self.title.trim().is_empty()
            && matches!(self.status, PlaybackStatus::Playing | PlaybackStatus::Paused)
    }

    pub fn position_at(&self, now: Ms) -> Option<Ms> {
        let t = self.timeline?;
        if t.end == 0 {
            return None;
        }
        let pos = if self.is_playing() {
            t.position + now.saturating_sub(t.reported_at)
        } else {
            t.position
        };
        Some(pos.min(t.end))
    }

    pub fn progress_at(&self, now: Ms) -> Option<f32> {
        let end = self.timeline?.end;
        self.position_at(now).map(|p| p as f32 / end.max(1) as f32)
    }

    pub fn source_name(&self) -> String {
        source_display_name(&self.source_app_id)
    }

    pub fn same_as(&self, other: &MediaSnapshot, now: Ms, tolerance: Ms) -> bool {
        let fields = self.source_app_id == other.source_app_id
            && self.title == other.title
            && self.artist == other.artist
            && self.album == other.album
            && self.status == other.status
            && self.controls == other.controls
            && self.session_index == other.session_index
            && self.session_count == other.session_count
            && self.track_id == other.track_id;
        let timeline = match (self.timeline, other.timeline) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.end == b.end
                    && match (self.position_at(now), other.position_at(now)) {
                        (Some(x), Some(y)) => x.abs_diff(y) < tolerance,
                        _ => a.position == b.position,
                    }
            }
            _ => false,
        };
        fields && timeline
    }
}

pub fn source_display_name(aumid: &str) -> String {
    let lower = aumid.to_ascii_lowercase();
    let known: &[(&str, &str)] = &[
        ("spotify", "Spotify"),
        ("applemusic", "Apple Music"),
        ("zunemusic", "Media Player"),
        ("msedge", "Edge"),
        ("chrome", "Chrome"),
        ("firefox", "Firefox"),
        ("brave", "Brave"),
        ("opera", "Opera"),
        ("vivaldi", "Vivaldi"),
        ("vlc", "VLC"),
        ("foobar2000", "foobar2000"),
        ("musicbee", "MusicBee"),
    ];
    if let Some((_, name)) = known.iter().find(|(k, _)| lower.contains(k)) {
        return (*name).to_string();
    }
    let base = aumid.split('!').next().unwrap_or(aumid);
    let base = base.split('_').next().unwrap_or(base);
    let base = base.strip_suffix(".exe").or_else(|| base.strip_suffix(".EXE")).unwrap_or(base);
    let base = base.rsplit(['\\', '/']).next().unwrap_or(base);
    let base = base.rsplit('.').next().unwrap_or(base);
    if base.is_empty() { "Media".to_string() } else { base.to_string() }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceFilter {
    pub allow: Vec<String>,
    pub block: Vec<String>,
}

impl SourceFilter {
    pub fn allows(&self, aumid: &str) -> bool {
        let id = aumid.to_ascii_lowercase();
        let hit = |list: &[String]| list.iter().any(|p| !p.is_empty() && id.contains(&p.to_ascii_lowercase()));
        if hit(&self.block) {
            return false;
        }
        self.allow.is_empty() || hit(&self.allow)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MediaPresence {
    paused_since: Option<Ms>,
    linger: Ms,
}

impl MediaPresence {
    pub fn new(linger: Ms) -> Self {
        Self { paused_since: None, linger }
    }

    pub fn update(&mut self, snap: Option<&MediaSnapshot>, now: Ms) {
        match snap {
            Some(s) if s.is_playing() => self.paused_since = None,
            Some(s) if s.has_content() => {
                self.paused_since.get_or_insert(now);
            }
            _ => self.paused_since = Some(0),
        }
    }

    pub fn is_present(&self, snap: Option<&MediaSnapshot>, now: Ms) -> bool {
        let Some(s) = snap.filter(|s| s.has_content()) else {
            return false;
        };
        s.is_playing() || self.paused_since.is_some_and(|t| now < t + self.linger)
    }

    pub fn deadline(&self) -> Option<Ms> {
        self.paused_since.filter(|&t| t > 0).map(|t| t + self.linger)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTimeline {
    pub position: Ms,
    pub end: Ms,
    pub updated: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TimelineAnchor {
    last: Option<(u64, RawTimeline, bool, Timeline)>,
}

impl TimelineAnchor {
    pub fn stabilize(&mut self, track_id: u64, raw: Option<RawTimeline>, fresh: Option<Timeline>, playing: bool, now: Ms) -> Option<Timeline> {
        let (Some(raw), Some(fresh)) = (raw, fresh) else {
            self.last = None;
            return fresh;
        };
        let timeline = match self.last {
            Some((id, prev_raw, was_playing, prev)) if id == track_id && prev_raw == raw => {
                if was_playing == playing {
                    prev
                } else {
                    let position = if was_playing {
                        (prev.position + now.saturating_sub(prev.reported_at)).min(prev.end)
                    } else {
                        prev.position
                    };
                    Timeline { position, end: prev.end, reported_at: now }
                }
            }
            _ => fresh,
        };
        self.last = Some((track_id, raw, playing, timeline));
        Some(timeline)
    }

    pub fn moved(&mut self, track_id: u64, position: Ms, now: Ms) {
        if let Some((id, _, _, tl)) = self.last.as_mut()
            && *id == track_id
        {
            *tl = Timeline { position: position.min(tl.end), end: tl.end, reported_at: now };
        }
    }
}

pub fn seek_target(snap: &MediaSnapshot, fraction: f32) -> Option<Ms> {
    let end = snap.timeline?.end;
    (end > 0 && snap.controls.seek).then(|| (end as f64 * fraction.clamp(0.0, 1.0) as f64).round() as Ms)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentHold {
    until: Option<Ms>,
}

impl ContentHold {
    pub fn hold(&mut self, had_content: bool, has_content: bool, now: Ms, window: Ms) -> Option<Ms> {
        if has_content || !had_content {
            self.until = None;
            return None;
        }
        let until = *self.until.get_or_insert(now + window);
        if now < until {
            Some(until)
        } else {
            self.until = None;
            None
        }
    }
}

pub fn art_fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)) ^ bytes.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(status: PlaybackStatus) -> MediaSnapshot {
        MediaSnapshot {
            title: "Lagu".into(),
            status,
            timeline: Some(Timeline { position: 10_000, end: 200_000, reported_at: 1_000 }),
            ..Default::default()
        }
    }

    #[test]
    fn laporan_posisi_rutin_dibuang() {
        let a = snap(PlaybackStatus::Playing);
        let mut b = a.clone();
        b.timeline = Some(Timeline { position: 15_200, end: 200_000, reported_at: 6_000 });
        assert!(a.same_as(&b, 6_000, 1_500));
        b.timeline = Some(Timeline { position: 90_000, end: 200_000, reported_at: 6_000 });
        assert!(!a.same_as(&b, 6_000, 1_500));
        assert!(!a.same_as(&snap(PlaybackStatus::Paused), 1_000, 1_500));
        let mut c = a.clone();
        c.title = "Lain".into();
        assert!(!a.same_as(&c, 1_000, 1_500));
    }

    #[test]
    fn posisi_diekstrapolasi_saat_diputar() {
        let s = snap(PlaybackStatus::Playing);
        assert_eq!(s.position_at(6_000), Some(15_000));
        assert_eq!(s.position_at(10_000_000), Some(200_000));
        let p = snap(PlaybackStatus::Paused);
        assert_eq!(p.position_at(6_000), Some(10_000));
    }

    #[test]
    fn nama_sumber() {
        assert_eq!(source_display_name("Spotify.exe"), "Spotify");
        assert_eq!(source_display_name("AppleInc.AppleMusicWin_nzyj5cx40ttqa!App"), "Apple Music");
        assert_eq!(source_display_name("MSEdge"), "Edge");
        assert_eq!(source_display_name("Chrome"), "Chrome");
        assert_eq!(source_display_name("Penerbit.PemutarKu_abc123!App"), "PemutarKu");
        assert_eq!(source_display_name("C:\\Apps\\tuner.exe"), "tuner");
        assert_eq!(source_display_name(""), "Media");
    }

    #[test]
    fn filter_sumber() {
        let f = SourceFilter { allow: vec![], block: vec!["chrome".into()] };
        assert!(!f.allows("Chrome"));
        assert!(f.allows("Spotify.exe"));
        let f = SourceFilter { allow: vec!["spotify".into()], block: vec![] };
        assert!(f.allows("Spotify.exe"));
        assert!(!f.allows("MSEdge"));
    }

    #[test]
    fn media_dijeda_bertahan_sebentar() {
        let mut p = MediaPresence::new(60_000);
        let playing = snap(PlaybackStatus::Playing);
        let paused = snap(PlaybackStatus::Paused);
        p.update(Some(&playing), 0);
        assert!(p.is_present(Some(&playing), 0));
        p.update(Some(&paused), 1_000);
        assert!(p.is_present(Some(&paused), 60_999));
        assert_eq!(p.deadline(), Some(61_000));
        assert!(!p.is_present(Some(&paused), 61_000));
        p.update(Some(&paused), 70_000);
        assert!(!p.is_present(Some(&paused), 70_000), "jeda tidak boleh diperpanjang");
        p.update(None, 80_000);
        assert!(!p.is_present(None, 80_000));
    }

    #[test]
    fn tombol_putar_mengikuti_kemampuan_sesi() {
        let only_toggle = Controls { toggle: true, ..Default::default() };
        assert!(only_toggle.can_play_pause(true) && only_toggle.can_play_pause(false));
        let play_only = Controls { play: true, ..Default::default() };
        assert!(play_only.can_play_pause(false));
        assert!(!play_only.can_play_pause(true));
        assert!(!Controls::default().can_play_pause(false));
    }

    #[test]
    fn posisi_tidak_melompat_saat_timeline_basi() {
        let raw = RawTimeline { position: 10_000, end: 200_000, updated: 7 };
        let fresh = |reported_at| Some(Timeline { position: 10_000, end: 200_000, reported_at });
        let mut a = TimelineAnchor::default();
        let tl = a.stabilize(1, Some(raw), fresh(1_000), true, 1_000).unwrap();
        assert_eq!(tl.reported_at, 1_000);

        let paused = a.stabilize(1, Some(raw), fresh(1_000), false, 31_000).unwrap();
        assert_eq!(paused.position, 40_000, "jeda membekukan posisi terakhir, bukan posisi laporan lama");

        let resumed = a.stabilize(1, Some(raw), fresh(1_000), true, 91_000).unwrap();
        assert_eq!(resumed.position, 40_000, "lanjut dari posisi jeda");
        let s = MediaSnapshot { status: PlaybackStatus::Playing, timeline: Some(resumed), ..Default::default() };
        assert_eq!(s.position_at(92_000), Some(41_000), "lama jeda tidak ikut dihitung");

        let still = a.stabilize(1, Some(raw), fresh(1_500), true, 95_000).unwrap();
        assert_eq!(still, resumed, "laporan yang sama tidak menggeser jangkar");
    }

    #[test]
    fn laporan_baru_dan_lagu_baru_memakai_timeline_segar() {
        let raw = RawTimeline { position: 10_000, end: 200_000, updated: 7 };
        let mut a = TimelineAnchor::default();
        a.stabilize(1, Some(raw), Some(Timeline { position: 10_000, end: 200_000, reported_at: 1_000 }), true, 1_000);
        let updated = RawTimeline { position: 50_000, updated: 9, ..raw };
        let fresh = Timeline { position: 50_000, end: 200_000, reported_at: 40_000 };
        assert_eq!(a.stabilize(1, Some(updated), Some(fresh), false, 41_000), Some(fresh));
        let next = Timeline { position: 0, end: 180_000, reported_at: 42_000 };
        assert_eq!(a.stabilize(2, Some(updated), Some(next), false, 42_000), Some(next));
        assert_eq!(a.stabilize(2, None, None, true, 43_000), None);
    }

    #[test]
    fn seek_menggeser_jangkar_dan_dibatasi_durasi() {
        let raw = RawTimeline { position: 10_000, end: 200_000, updated: 7 };
        let mut a = TimelineAnchor::default();
        a.stabilize(3, Some(raw), Some(Timeline { position: 10_000, end: 200_000, reported_at: 0 }), true, 0);
        a.moved(3, 120_000, 5_000);
        let tl = a.stabilize(3, Some(raw), Some(Timeline { position: 10_000, end: 200_000, reported_at: 0 }), true, 6_000).unwrap();
        assert_eq!(tl, Timeline { position: 120_000, end: 200_000, reported_at: 5_000 }, "laporan basi tidak menarik balik posisi");
        a.moved(9, 1, 7_000);
        a.moved(3, 999_999, 7_000);
        let tl = a.stabilize(3, Some(raw), Some(tl), true, 7_000).unwrap();
        assert_eq!(tl.position, 200_000, "seek ke lagu lain diabaikan, posisi dibatasi durasi");

        let mut s = snap(PlaybackStatus::Playing);
        assert_eq!(seek_target(&s, 0.5), None, "sesi tanpa dukungan seek");
        s.controls.seek = true;
        assert_eq!(seek_target(&s, 0.5), Some(100_000));
        assert_eq!(seek_target(&s, 7.0), Some(200_000));
        assert_eq!(seek_target(&s, -1.0), Some(0));
        s.timeline = None;
        assert_eq!(seek_target(&s, 0.5), None);
    }

    #[test]
    fn jeda_singkat_antar_lagu_ditahan() {
        let mut h = ContentHold::default();
        assert_eq!(h.hold(true, false, 1_000, 1_500), Some(2_500));
        assert_eq!(h.hold(true, false, 2_000, 1_500), Some(2_500), "batas waktu tidak diperpanjang");
        assert_eq!(h.hold(true, false, 2_500, 1_500), None, "lewat batas: kosong diteruskan");
        assert_eq!(h.hold(true, false, 3_000, 1_500), Some(4_500));
        assert_eq!(h.hold(true, true, 3_100, 1_500), None, "isi kembali: tahanan dilepas");
        assert_eq!(h.hold(false, false, 3_200, 1_500), None, "tidak ada yang perlu ditahan");
    }

    #[test]
    fn sidik_sampul_membedakan_isi() {
        assert_eq!(art_fingerprint(b"abc"), art_fingerprint(b"abc"));
        assert_ne!(art_fingerprint(b"abc"), art_fingerprint(b"abd"));
        assert_ne!(art_fingerprint(&[]), art_fingerprint(&[0]));
    }

    #[test]
    fn tanpa_judul_bukan_aktivitas() {
        let mut s = snap(PlaybackStatus::Playing);
        s.title = "  ".into();
        assert!(!s.has_content());
        assert!(!MediaPresence::new(1).is_present(Some(&s), 0));
    }
}
