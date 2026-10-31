use std::ffi::c_void;

use notchi_core::lyrics::{self, Lyrics, Query};
use windows::Win32::Networking::WinHttp::{
    INTERNET_DEFAULT_HTTPS_PORT, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_DISABLE_COOKIES, WINHTTP_FLAG_SECURE,
    WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3, WINHTTP_OPTION_DISABLE_FEATURE,
    WINHTTP_OPTION_SECURE_PROTOCOLS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE, WinHttpCloseHandle,
    WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable, WinHttpQueryHeaders, WinHttpReadData,
    WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetOption, WinHttpSetTimeouts,
};
use windows::Win32::Foundation::{E_FAIL, E_OUTOFMEMORY};
use windows_core::{Error, HSTRING, PCWSTR, Result};

pub const LYRICS_HOST: &str = "lrclib.net";
pub const LYRICS_FALLBACK_HOST: &str = "api.lyrics.ovh";

struct Handle(*mut c_void);

impl Handle {
    fn new(raw: *mut c_void) -> Result<Handle> {
        if raw.is_null() { Err(Error::from_thread()) } else { Ok(Handle(raw)) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u32,
    pub body: Vec<u8>,
}

fn user_agent() -> String {
    format!("Notchi/{} (+https://github.com/devaldicaliesta/devnotch)", env!("CARGO_PKG_VERSION"))
}

pub fn https_get(host: &str, path: &str, max_bytes: usize) -> Result<Response> {
    let agent = HSTRING::from(user_agent());
    let host = HSTRING::from(host);
    let path = HSTRING::from(path);
    unsafe {
        let session = Handle::new(WinHttpOpen(&agent, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, PCWSTR::null(), PCWSTR::null(), 0))?;
        let protocols = (WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3).to_le_bytes();
        let _ = WinHttpSetOption(Some(session.0), WINHTTP_OPTION_SECURE_PROTOCOLS, Some(&protocols));
        WinHttpSetTimeouts(session.0, 5_000, 5_000, 8_000, 8_000)?;
        let connect = Handle::new(WinHttpConnect(session.0, &host, INTERNET_DEFAULT_HTTPS_PORT, 0))?;
        let request = Handle::new(WinHttpOpenRequest(
            connect.0,
            &HSTRING::from("GET"),
            &path,
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        ))?;
        let no_cookies = WINHTTP_DISABLE_COOKIES.to_le_bytes();
        let _ = WinHttpSetOption(Some(request.0), WINHTTP_OPTION_DISABLE_FEATURE, Some(&no_cookies));
        let headers: Vec<u16> = "Accept: application/json\r\n".encode_utf16().collect();
        WinHttpSendRequest(request.0, Some(&headers), None, 0, 0, 0)?;
        WinHttpReceiveResponse(request.0, std::ptr::null_mut())?;
        let mut status = 0u32;
        let mut len = size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut c_void),
            &mut len,
            std::ptr::null_mut(),
        )?;
        let mut body = Vec::new();
        loop {
            let mut available = 0u32;
            WinHttpQueryDataAvailable(request.0, &mut available)?;
            if available == 0 {
                break;
            }
            if body.len() + available as usize > max_bytes {
                return Err(Error::from_hresult(E_OUTOFMEMORY));
            }
            let start = body.len();
            body.resize(start + available as usize, 0);
            let mut read = 0u32;
            WinHttpReadData(request.0, body[start..].as_mut_ptr() as *mut c_void, available, &mut read)?;
            body.truncate(start + read as usize);
            if read == 0 {
                break;
            }
        }
        Ok(Response { status, body })
    }
}

#[derive(Debug, Clone, Default)]
pub struct LyricsFetch {
    pub lyrics: Option<Lyrics>,
    pub retry: bool,
}

fn json_from(host: &str, path: &str) -> Result<Option<lyrics::Json>> {
    let r = https_get(host, path, lyrics::MAX_RESPONSE_BYTES)?;
    match r.status {
        200 => Ok(lyrics::parse_json(&String::from_utf8_lossy(&r.body))),
        404 => Ok(None),
        status => Err(Error::new(E_FAIL, format!("{host} HTTP {status}"))),
    }
}

fn lrclib(q: &Query) -> Result<Option<Lyrics>> {
    let pick = |path: String| -> Result<Option<Lyrics>> { Ok(json_from(LYRICS_HOST, &path)?.and_then(|j| lyrics::pick(&j, q.duration))) };
    if !q.artist.is_empty()
        && let Some(l) = pick(q.get_path())?
    {
        return Ok(Some(l));
    }
    pick(q.search_path())
}

fn lyrics_ovh(q: &Query) -> Result<Option<Lyrics>> {
    let Some(path) = q.ovh_path() else { return Ok(None) };
    Ok(json_from(LYRICS_FALLBACK_HOST, &path)?.and_then(|j| lyrics::parse_ovh(&j)))
}

pub fn fetch_lyrics(q: &Query) -> LyricsFetch {
    if !q.is_usable() {
        return LyricsFetch::default();
    }
    let primary = lrclib(q);
    if let Ok(Some(l)) = primary {
        return LyricsFetch { lyrics: Some(l), retry: false };
    }
    let fallback = lyrics_ovh(q);
    let retry = primary.is_err() || fallback.is_err();
    LyricsFetch { lyrics: fallback.ok().flatten(), retry }
}
