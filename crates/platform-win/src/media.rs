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
    Quick,
    TogglePlayPause,
    Next,
    Prev,
    CycleSession(i32),
    SetFilter(SourceFilter),
    Shutdown,
}

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
                let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
                run(proxy, filter, tx2, rx);
                if com {
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
    preferred: Option<String>,
    props: Option<(String, String, String)>,
    position: (usize, usize),
    last_track: Option<(String, String, String)>,
    last_sent: Option<MediaSnapshot>,
    track_id: u64,
}

const POSITION_TOLERANCE_MS: u64 = 1_500;

fn refresh_handler<S: windows_core::RuntimeType + 'static, A: windows_core::RuntimeType + 'static>(
    tx: &Sender<MediaCommand>,
    cmd: MediaCommand,
) -> TypedEventHandler<S, A> {
    let tx = tx.clone();
    TypedEventHandler::new(move |_, _| {
        let _ = tx.send(cmd.clone());
        Ok(())
    })
}

fn run(proxy: UiProxy, filter: SourceFilter, tx: Sender<MediaCommand>, rx: Receiver<MediaCommand>) {
    let manager = match Manager::RequestAsync().and_then(|op| op.join()) {
        Ok(m) => m,
        Err(_) => {
            proxy.post(Event::Media(None));
            while let Ok(cmd) = rx.recv() {
                if matches!(cmd, MediaCommand::Shutdown) {
                    return;
                }
            }
            return;
        }
    };
    let t1 = manager.SessionsChanged(&refresh_handler(&tx, MediaCommand::Refresh)).ok();
    let t2 = manager.CurrentSessionChanged(&refresh_handler(&tx, MediaCommand::Refresh)).ok();

    let mut st = State {
        proxy,
        filter,
        tx,
        subscribed: None,
        preferred: None,
        props: None,
        position: (0, 0),
        last_track: None,
        last_sent: None,
        track_id: 0,
    };
    st.refresh(&manager, true);

    while let Ok(first) = rx.recv() {
        let mut pending = Some(first);
        let mut full = false;
        let mut quick = false;
        while let Some(cmd) = pending.take().or_else(|| rx.try_recv().ok()) {
            match cmd {
                MediaCommand::Refresh => full = true,
                MediaCommand::Quick => quick = true,
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
                    full = true;
                }
                MediaCommand::SetFilter(f) => {
                    st.filter = f;
                    st.preferred = None;
                    full = true;
                }
                MediaCommand::Shutdown => {
                    st.shutdown(&manager, t1, t2);
                    return;
                }
            }
        }
        if full || quick {
            st.refresh(&manager, full);
        }
    }
    st.shutdown(&manager, t1, t2);
}

fn aumid(s: &Session) -> String {
    s.SourceAppUserModelId().map(|h| h.to_string()).unwrap_or_default()
}

fn read_props(session: &Session) -> (String, String, String) {
    match session.TryGetMediaPropertiesAsync().and_then(|op| op.join()) {
        Ok(p) => (
            p.Title().map(|s| s.to_string()).unwrap_or_default(),
            p.Artist().map(|s| s.to_string()).unwrap_or_default(),
            p.AlbumTitle().map(|s| s.to_string()).unwrap_or_default(),
        ),
        Err(_) => Default::default(),
    }
}

impl State {
    fn shutdown(&mut self, manager: &Manager, t1: Option<i64>, t2: Option<i64>) {
        if let Some(s) = self.subscribed.take() {
            s.unsubscribe();
        }
        if let Some(t) = t1 {
            let _ = manager.RemoveSessionsChanged(t);
        }
        if let Some(t) = t2 {
            let _ = manager.RemoveCurrentSessionChanged(t);
        }
    }

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
        if let Ok(cur) = manager.GetCurrentSession() {
            let id = aumid(&cur);
            if let Some(i) = sessions.iter().position(|s| aumid(s) == id) {
                return (Some(sessions[i].clone()), i, count);
            }
        }
        (sessions.first().cloned(), 0, count)
    }

    fn subscribe(&mut self, session: &Session) -> bool {
        if self.subscribed.as_ref().is_some_and(|s| s.session == *session) {
            return false;
        }
        if let Some(old) = self.subscribed.take() {
            old.unsubscribe();
        }
        let t0 = session.MediaPropertiesChanged(&refresh_handler(&self.tx, MediaCommand::Refresh)).unwrap_or(0);
        let t1 = session.PlaybackInfoChanged(&refresh_handler(&self.tx, MediaCommand::Quick)).unwrap_or(0);
        let t2 = session.TimelinePropertiesChanged(&refresh_handler(&self.tx, MediaCommand::Quick)).unwrap_or(0);
        self.subscribed = Some(Subscribed { session: session.clone(), tokens: [t0, t1, t2] });
        true
    }

    fn post(&mut self, snap: Option<MediaSnapshot>) {
        let same = match (&self.last_sent, &snap) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same_as(b, now_ms(), POSITION_TOLERANCE_MS),
            _ => false,
        };
        if !same {
            self.proxy.post(Event::Media(snap.clone()));
            self.last_sent = snap;
        }
    }

    fn refresh(&mut self, manager: &Manager, full: bool) {
        let current = self.subscribed.as_ref().map(|s| s.session.clone()).filter(|_| !full);
        let (session, index, count) = match current {
            Some(s) => (Some(s), self.position.0, self.position.1),
            None => self.choose(manager),
        };
        let Some(session) = session else {
            if let Some(old) = self.subscribed.take() {
                old.unsubscribe();
            }
            self.last_track = None;
            self.props = None;
            self.post(None);
            return;
        };
        self.position = (index, count);
        let switched = self.subscribe(&session);
        if full || switched || self.props.is_none() {
            self.props = Some(read_props(&session));
        }
        let mut snap = self.snapshot(&session, index, count);
        let key = (snap.source_app_id.clone(), snap.title.clone(), snap.artist.clone());
        let changed = self.last_track.as_ref() != Some(&key);
        if changed {
            self.track_id += 1;
            self.last_track = Some(key);
        }
        snap.track_id = self.track_id;
        self.post(Some(snap));
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
        if let Some((title, artist, album)) = &self.props {
            snap.title = title.clone();
            snap.artist = artist.clone();
            snap.album = album.clone();
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

fn windows_filetime_now() -> i64 {
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
    decode_artwork(&buf, 192).ok()
}
