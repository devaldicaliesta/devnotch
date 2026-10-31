//! Sesi media dari kontrol media sistem Windows (PRD §3.2, §3.6).
//!
//! Berjalan di thread MTA sendiri. Event WinRT hanya memicu `Refresh`
//! (digabung bila beruntun), lalu snapshot lengkap dikirim ke thread UI.
//! Aplikasi tidak membaca isi layar, tidak menyadap jaringan, dan tidak
//! menyuntik skrip ke browser: hanya sesi yang diumumkan ke Windows yang
//! tampil.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

use notchi_core::media::{Controls, MediaSnapshot, PlaybackStatus, SourceFilter, Timeline};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as WinStatus,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

use crate::gfx::{MAX_ART_BYTES, decode_artwork};
use crate::now_ms;
use crate::window::{Event, UiProxy};

#[derive(Debug, Clone)]
pub enum MediaCommand {
    Refresh,
    TogglePlayPause,
    Next,
    Prev,
    /// Berpindah ke sesi lain (geser di Island).
    CycleSession(i32),
    SetFilter(SourceFilter),
    Shutdown,
}

/// Pegangan layanan media; menghentikan thread saat di-drop.
#[derive(Debug)]
pub struct MediaService {
    tx: Sender<MediaCommand>,
    thread: Option<JoinHandle<()>>,
}

impl MediaService {
    pub fn start(proxy: UiProxy, filter: SourceFilter) -> MediaService {
        let (tx, rx) = channel();
        let tx2 = tx.clone();
        let thread = std::thread::Builder::new()
            .name("notchi-media".into())
            .spawn(move || {
                // SAFETY: inisialisasi COM MTA untuk thread ini, dilepas di akhir.
                let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
                run(proxy, filter, tx2, rx);
                if com {
                    // SAFETY: berpasangan dengan CoInitializeEx di atas.
                    unsafe { CoUninitialize() };
                }
            })
            .ok();
        MediaService { tx, thread }
    }

    pub fn send(&self, cmd: MediaCommand) {
        let _ = self.tx.send(cmd);
    }
}

impl Drop for MediaService {
    fn drop(&mut self) {
        let _ = self.tx.send(MediaCommand::Shutdown);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

struct Subscribed {
    session: Session,
    tokens: [i64; 3],
}

impl Subscribed {
    fn unsubscribe(self) {
        let s = &self.session;
        let _ = s.RemoveMediaPropertiesChanged(self.tokens[0]);
        let _ = s.RemovePlaybackInfoChanged(self.tokens[1]);
        let _ = s.RemoveTimelinePropertiesChanged(self.tokens[2]);
    }
}

struct State {
    proxy: UiProxy,
    filter: SourceFilter,
    tx: Sender<MediaCommand>,
    subscribed: Option<Subscribed>,
    /// AUMID yang dipilih pengguna lewat geser; `None` = ikuti sistem.
    preferred: Option<String>,
    last_track: Option<(String, String, String)>,
    track_id: u64,
}

fn refresh_handler<S: windows_core::RuntimeType + 'static, A: windows_core::RuntimeType + 'static>(
    tx: &Sender<MediaCommand>,
) -> TypedEventHandler<S, A> {
    let tx = tx.clone();
    TypedEventHandler::new(move |_, _| {
        let _ = tx.send(MediaCommand::Refresh);
        Ok(())
    })
}

fn run(proxy: UiProxy, filter: SourceFilter, tx: Sender<MediaCommand>, rx: Receiver<MediaCommand>) {
    let manager = match Manager::RequestAsync().and_then(|op| op.join()) {
        Ok(m) => m,
        Err(_) => {
            proxy.post(Event::Media(None));
            // Tetap melayani perintah agar pemanggil tidak terblokir.
            while let Ok(cmd) = rx.recv() {
                if matches!(cmd, MediaCommand::Shutdown) {
                    return;
                }
            }
            return;
        }
    };
    let t1 = manager.SessionsChanged(&refresh_handler(&tx)).ok();
    let t2 = manager.CurrentSessionChanged(&refresh_handler(&tx)).ok();

    let mut st = State { proxy, filter, tx, subscribed: None, preferred: None, last_track: None, track_id: 0 };
    let _ = st.tx.send(MediaCommand::Refresh);

    while let Ok(mut cmd) = rx.recv() {
        // Gabungkan Refresh beruntun menjadi satu.
        if matches!(cmd, MediaCommand::Refresh) {
            while let Ok(next) = rx.try_recv() {
                if !matches!(next, MediaCommand::Refresh) {
                    st.refresh(&manager);
                    cmd = next;
                    break;
                }
            }
        }
        match cmd {
            MediaCommand::Refresh => st.refresh(&manager),
            MediaCommand::TogglePlayPause => st.with_session(|s| {
                let _ = s.TryTogglePlayPauseAsync().and_then(|op| op.join());
            }),
            MediaCommand::Next => st.with_session(|s| {
                let _ = s.TrySkipNextAsync().and_then(|op| op.join());
            }),
            MediaCommand::Prev => st.with_session(|s| {
                let _ = s.TrySkipPreviousAsync().and_then(|op| op.join());
            }),
            MediaCommand::CycleSession(step) => {
                st.cycle(&manager, step);
                st.refresh(&manager);
            }
            MediaCommand::SetFilter(f) => {
                st.filter = f;
                st.preferred = None;
                st.refresh(&manager);
            }
            MediaCommand::Shutdown => break,
        }
    }
    if let Some(s) = st.subscribed.take() {
        s.unsubscribe();
    }
    if let Some(t) = t1 {
        let _ = manager.RemoveSessionsChanged(t);
    }
    if let Some(t) = t2 {
        let _ = manager.RemoveCurrentSessionChanged(t);
    }
}

fn aumid(s: &Session) -> String {
    s.SourceAppUserModelId().map(|h| h.to_string()).unwrap_or_default()
}

impl State {
    fn with_session(&self, f: impl FnOnce(&Session)) {
        if let Some(s) = &self.subscribed {
            f(&s.session);
        }
    }

    fn allowed_sessions(&self, manager: &Manager) -> Vec<Session> {
        let Ok(list) = manager.GetSessions() else { return Vec::new() };
        list.into_iter().filter(|s| self.filter.allows(&aumid(s))).collect()
    }

    fn cycle(&mut self, manager: &Manager, step: i32) {
        let sessions = self.allowed_sessions(manager);
        if sessions.len() < 2 {
            return;
        }
        let cur = self.subscribed.as_ref().map(|s| aumid(&s.session));
        let idx = sessions.iter().position(|s| Some(aumid(s)) == cur).unwrap_or(0) as i64;
        let next = (idx + step as i64).rem_euclid(sessions.len() as i64) as usize;
        self.preferred = Some(aumid(&sessions[next]));
    }

    fn choose(&mut self, manager: &Manager) -> (Option<Session>, usize, usize) {
        let sessions = self.allowed_sessions(manager);
        let count = sessions.len();
        if let Some(p) = &self.preferred {
            if let Some(i) = sessions.iter().position(|s| &aumid(s) == p) {
                return (Some(sessions[i].clone()), i, count);
            }
            self.preferred = None;
        }
        // Sesi yang dinilai sistem paling mungkin dikontrol pengguna.
        if let Ok(cur) = manager.GetCurrentSession() {
            let id = aumid(&cur);
            if let Some(i) = sessions.iter().position(|s| aumid(s) == id) {
                return (Some(sessions[i].clone()), i, count);
            }
        }
        (sessions.first().cloned(), 0, count)
    }

    fn subscribe(&mut self, session: &Session) {
        let same = self.subscribed.as_ref().is_some_and(|s| aumid(&s.session) == aumid(session) && s.session == *session);
        if same {
            return;
        }
        if let Some(old) = self.subscribed.take() {
            old.unsubscribe();
        }
        let t0 = session.MediaPropertiesChanged(&refresh_handler(&self.tx)).unwrap_or(0);
        let t1 = session.PlaybackInfoChanged(&refresh_handler(&self.tx)).unwrap_or(0);
        let t2 = session.TimelinePropertiesChanged(&refresh_handler(&self.tx)).unwrap_or(0);
        self.subscribed = Some(Subscribed { session: session.clone(), tokens: [t0, t1, t2] });
    }

    fn refresh(&mut self, manager: &Manager) {
        let (session, index, count) = self.choose(manager);
        let Some(session) = session else {
            if let Some(old) = self.subscribed.take() {
                old.unsubscribe();
            }
            self.last_track = None;
            self.proxy.post(Event::Media(None));
            return;
        };
        self.subscribe(&session);
        let snap = self.snapshot(&session, index, count);
        let key = (snap.source_app_id.clone(), snap.title.clone(), snap.artist.clone());
        let changed = self.last_track.as_ref() != Some(&key);
        if changed {
            self.track_id += 1;
            self.last_track = Some(key);
        }
        let mut snap = snap;
        snap.track_id = self.track_id;
        self.proxy.post(Event::Media(Some(snap)));
        if changed {
            let art = load_artwork(&session);
            self.proxy.post(Event::MediaArt { track_id: self.track_id, art });
        }
    }

    fn snapshot(&self, session: &Session, index: usize, count: usize) -> MediaSnapshot {
        let mut snap = MediaSnapshot {
            source_app_id: aumid(session),
            session_index: index,
            session_count: count,
            ..Default::default()
        };
        if let Ok(props) = session.TryGetMediaPropertiesAsync().and_then(|op| op.join()) {
            snap.title = props.Title().map(|s| s.to_string()).unwrap_or_default();
            snap.artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
            snap.album = props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();
        }
        if let Ok(info) = session.GetPlaybackInfo() {
            snap.status = match info.PlaybackStatus() {
                Ok(WinStatus::Opened) => PlaybackStatus::Opened,
                Ok(WinStatus::Changing) => PlaybackStatus::Changing,
                Ok(WinStatus::Stopped) => PlaybackStatus::Stopped,
                Ok(WinStatus::Playing) => PlaybackStatus::Playing,
                Ok(WinStatus::Paused) => PlaybackStatus::Paused,
                _ => PlaybackStatus::Closed,
            };
            if let Ok(c) = info.Controls() {
                snap.controls = Controls {
                    play: c.IsPlayEnabled().unwrap_or(false),
                    pause: c.IsPauseEnabled().unwrap_or(false),
                    next: c.IsNextEnabled().unwrap_or(false),
                    prev: c.IsPreviousEnabled().unwrap_or(false),
                };
            }
        }
        if let Ok(tl) = session.GetTimelineProperties() {
            let ms = |t: windows::Foundation::TimeSpan| (t.Duration.max(0) / 10_000) as u64;
            let start = tl.StartTime().map(ms).unwrap_or(0);
            let end = tl.EndTime().map(ms).unwrap_or(0);
            let pos = tl.Position().map(ms).unwrap_or(0);
            // Umur laporan posisi: sekarang dikurangi LastUpdatedTime.
            let age = tl
                .LastUpdatedTime()
                .ok()
                .map(|dt| {
                    let now = windows_filetime_now();
                    ((now - dt.UniversalTime).max(0) / 10_000) as u64
                })
                .unwrap_or(0)
                .min(24 * 3600 * 1000);
            if end > start {
                snap.timeline = Some(Timeline {
                    position: pos.saturating_sub(start),
                    end: end - start,
                    reported_at: now_ms().saturating_sub(age),
                });
            }
        }
        snap
    }
}

/// Waktu sekarang dalam satuan 100 ns sejak 1601 (format `DateTime` WinRT).
fn windows_filetime_now() -> i64 {
    // SAFETY: fungsi tanpa argumen yang hanya membaca jam sistem.
    let ft = unsafe { windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime() };
    ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64
}

fn load_artwork(session: &Session) -> Option<notchi_core::media::Artwork> {
    let props = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let thumb = props.Thumbnail().ok()?;
    let stream = thumb.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()? as usize;
    if size == 0 || size > MAX_ART_BYTES {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let loaded = reader.LoadAsync(size as u32).ok()?.join().ok()? as usize;
    let mut buf = vec![0u8; loaded];
    reader.ReadBytes(&mut buf).ok()?;
    // Sampul tidak pernah disimpan ke disk; hanya versi kecil di memori.
    decode_artwork(&buf, 192).ok()
}
