use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use notchi_core::Ms;
use notchi_core::media::{
    ContentHold, Controls, MediaSnapshot, PlaybackStatus, RawTimeline, SourceFilter, Timeline, TimelineAnchor, art_fingerprint,
};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as WinStatus,
};
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

use crate::gfx::{MAX_ART_BYTES, decode_artwork};
use crate::now_ms;
use crate::window::{Event, UiProxy};

#[derive(Debug, Clone)]
pub enum MediaCommand {
    Refresh,
    Quick,
    CurrentChanged,
    TogglePlayPause,
    Next,
    Prev,
    Seek(Ms),
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

struct Props {
    title: String,
    artist: String,
    album: String,
    thumbnail: Option<IRandomAccessStreamReference>,
}

struct State {
    proxy: UiProxy,
    filter: SourceFilter,
    tx: Sender<MediaCommand>,
    subscribed: Option<Subscribed>,
    preferred: Option<String>,
    props: Option<Props>,
    position: (usize, usize),
    last_track: Option<(String, String, String)>,
    last_sent: Option<MediaSnapshot>,
    track_id: u64,
    art_print: Option<u64>,
    anchor: TimelineAnchor,
    hold: ContentHold,
    wake_at: Option<Ms>,
    timeline_start: Ms,
    force_post: bool,
}

const POSITION_TOLERANCE_MS: u64 = 1_500;
const HOLD_MS: Ms = 1_500;
const TRACK_SETTLE_MS: Ms = 1_200;
const COMMAND_SETTLE_MS: Ms = 400;

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

fn next_command(rx: &Receiver<MediaCommand>, wake_at: Option<Ms>) -> Result<Option<MediaCommand>, ()> {
    match wake_at {
        Some(at) => match rx.recv_timeout(Duration::from_millis(at.saturating_sub(now_ms()))) {
            Ok(cmd) => Ok(Some(cmd)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(()),
        },
        None => rx.recv().map(Some).map_err(|_| ()),
    }
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
    let t2 = manager.CurrentSessionChanged(&refresh_handler(&tx, MediaCommand::CurrentChanged)).ok();

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
        art_print: None,
        anchor: TimelineAnchor::default(),
        hold: ContentHold::default(),
        wake_at: None,
        timeline_start: 0,
        force_post: false,
    };
    st.refresh(&manager, true);

    while let Ok(first) = next_command(&rx, st.wake_at) {
        let mut full = false;
        let mut quick = false;
        if st.wake_at.is_some_and(|at| now_ms() >= at) {
            st.wake_at = None;
            full = true;
        }
        let mut pending = first;
        while let Some(cmd) = pending.take().or_else(|| rx.try_recv().ok()) {
            match cmd {
                MediaCommand::Refresh => full = true,
                MediaCommand::Quick => quick = true,
                MediaCommand::CurrentChanged => {
                    st.follow_current(&manager);
                    full = true;
                }
                MediaCommand::TogglePlayPause => {
                    st.with_session(play_pause);
                    st.settle_after_command();
                    quick = true;
                }
                MediaCommand::Next => {
                    st.with_session(|s| {
                        let _ = s.TrySkipNextAsync().and_then(|op| op.join());
                    });
                    st.settle_after_command();
                    quick = true;
                }
                MediaCommand::Prev => {
                    st.with_session(|s| {
                        let _ = s.TrySkipPreviousAsync().and_then(|op| op.join());
                    });
                    st.settle_after_command();
                    quick = true;
                }
                MediaCommand::Seek(position) => {
                    st.seek(position);
                    st.force_post = true;
                    st.settle_after_command();
                    quick = true;
                }
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

fn status_of(s: &Session) -> Option<WinStatus> {
    s.GetPlaybackInfo().and_then(|i| i.PlaybackStatus()).ok()
}

fn play_pause(s: &Session) {
    let direct = if status_of(s) == Some(WinStatus::Playing) { s.TryPauseAsync() } else { s.TryPlayAsync() };
    if !direct.and_then(|op| op.join()).unwrap_or(false) {
        let _ = s.TryTogglePlayPauseAsync().and_then(|op| op.join());
    }
}

fn read_props(session: &Session) -> Props {
    match session.TryGetMediaPropertiesAsync().and_then(|op| op.join()) {
        Ok(p) => Props {
            title: p.Title().map(|s| s.to_string()).unwrap_or_default(),
            artist: p.Artist().map(|s| s.to_string()).unwrap_or_default(),
            album: p.AlbumTitle().map(|s| s.to_string()).unwrap_or_default(),
            thumbnail: p.Thumbnail().ok(),
        },
        Err(_) => Props { title: String::new(), artist: String::new(), album: String::new(), thumbnail: None },
    }
}

fn read_thumbnail(thumb: &IRandomAccessStreamReference) -> Option<Vec<u8>> {
    let stream = thumb.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()? as usize;
    if size == 0 || size > MAX_ART_BYTES {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let loaded = reader.LoadAsync(size as u32).ok()?.join().ok()? as usize;
    let mut buf = vec![0u8; loaded];
    reader.ReadBytes(&mut buf).ok()?;
    Some(buf)
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

    fn wake(&mut self, at: Ms) {
        self.wake_at = Some(self.wake_at.map_or(at, |w| w.min(at)));
    }

    fn settle_after_command(&mut self) {
        self.wake(now_ms() + COMMAND_SETTLE_MS);
    }

    fn seek(&mut self, position: Ms) {
        let Some(sub) = &self.subscribed else { return };
        let ticks = (self.timeline_start + position).saturating_mul(10_000).min(i64::MAX as u64) as i64;
        let moved = sub.session.TryChangePlaybackPositionAsync(ticks).and_then(|op| op.join()).unwrap_or(false);
        if moved {
            self.anchor.moved(self.track_id, position, now_ms());
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

    fn follow_current(&mut self, manager: &Manager) {
        let Some(preferred) = &self.preferred else { return };
        let Ok(current) = manager.GetCurrentSession() else { return };
        if status_of(&current) == Some(WinStatus::Playing) && aumid(&current) != *preferred {
            self.preferred = None;
        }
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
        let forced = std::mem::take(&mut self.force_post);
        if !same || forced {
            self.proxy.post(Event::Media(snap.clone()));
            self.last_sent = snap;
        }
    }

    fn held(&mut self, has_content: bool, now: Ms) -> bool {
        let had_content = self.last_sent.as_ref().is_some_and(|s| s.has_content());
        match self.hold.hold(had_content, has_content, now, HOLD_MS) {
            Some(until) => {
                self.wake(until);
                true
            }
            None => false,
        }
    }

    fn refresh(&mut self, manager: &Manager, full: bool) {
        let now = now_ms();
        let current = self.subscribed.as_ref().map(|s| s.session.clone()).filter(|_| !full);
        let (session, index, count) = match current {
            Some(s) => (Some(s), self.position.0, self.position.1),
            None => self.choose(manager),
        };
        let Some(session) = session else {
            if self.held(false, now) {
                return;
            }
            if let Some(old) = self.subscribed.take() {
                old.unsubscribe();
            }
            self.last_track = None;
            self.props = None;
            self.art_print = None;
            self.anchor = TimelineAnchor::default();
            self.post(None);
            return;
        };
        self.position = (index, count);
        let switched = self.subscribe(&session);
        if full || switched || self.props.is_none() {
            self.props = Some(read_props(&session));
        }
        let (mut snap, raw, start) = self.snapshot(&session, index, count, now);
        if self.held(snap.has_content(), now) {
            return;
        }
        self.timeline_start = start;
        let key = (snap.source_app_id.clone(), snap.title.clone(), snap.artist.clone());
        let changed = self.last_track.as_ref() != Some(&key);
        if changed {
            self.track_id += 1;
            self.last_track = Some(key);
            self.wake(now + TRACK_SETTLE_MS);
        }
        snap.track_id = self.track_id;
        snap.timeline = self.anchor.stabilize(snap.track_id, raw, snap.timeline, snap.is_playing(), now);
        self.post(Some(snap));
        if changed || full {
            self.update_art(changed);
        }
    }

    fn update_art(&mut self, changed: bool) {
        let bytes = self.props.as_ref().and_then(|p| p.thumbnail.as_ref()).and_then(read_thumbnail);
        let print = bytes.as_deref().map(art_fingerprint);
        if !changed && (print.is_none() || print == self.art_print) {
            return;
        }
        self.art_print = print;
        let art = bytes.and_then(|b| decode_artwork(&b, 192).ok());
        self.proxy.post(Event::MediaArt { track_id: self.track_id, art });
    }

    fn snapshot(&self, session: &Session, index: usize, count: usize, now: Ms) -> (MediaSnapshot, Option<RawTimeline>, Ms) {
        let mut snap = MediaSnapshot {
            source_app_id: aumid(session),
            session_index: index,
            session_count: count,
            ..Default::default()
        };
        if let Some(p) = &self.props {
            snap.title = p.title.clone();
            snap.artist = p.artist.clone();
            snap.album = p.album.clone();
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
                    toggle: c.IsPlayPauseToggleEnabled().unwrap_or(false),
                    next: c.IsNextEnabled().unwrap_or(false),
                    prev: c.IsPreviousEnabled().unwrap_or(false),
                    seek: c.IsPlaybackPositionEnabled().unwrap_or(false),
                };
            }
        }
        let mut raw = None;
        let mut origin = 0;
        if let Ok(tl) = session.GetTimelineProperties() {
            let ms = |t: windows::Foundation::TimeSpan| (t.Duration.max(0) / 10_000) as u64;
            let start = tl.StartTime().map(ms).unwrap_or(0);
            let end = tl.EndTime().map(ms).unwrap_or(0);
            let pos = tl.Position().map(ms).unwrap_or(0);
            let updated = tl.LastUpdatedTime().map(|dt| dt.UniversalTime).unwrap_or(0);
            if end > start {
                let length = end - start;
                let age = (updated > 0)
                    .then(|| ((windows_filetime_now() - updated).max(0) / 10_000) as u64)
                    .filter(|&age| age <= length)
                    .unwrap_or(0);
                snap.timeline = Some(Timeline {
                    position: pos.saturating_sub(start).min(length),
                    end: length,
                    reported_at: now.saturating_sub(age),
                });
                raw = Some(RawTimeline { position: pos, end, updated });
                origin = start;
            }
        }
        (snap, raw, origin)
    }
}

fn windows_filetime_now() -> i64 {
    let ft = unsafe { windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime() };
    ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64
}
