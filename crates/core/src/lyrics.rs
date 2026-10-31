use crate::Ms;

pub const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_LINES: usize = 2_000;
const MAX_LINE_CHARS: usize = 300;
const MAX_DEPTH: u32 = 16;
const DURATION_SLACK_MS: Ms = 4_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub at: Ms,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lyrics {
    pub lines: Vec<Line>,
    pub synced: bool,
    pub instrumental: bool,
}

fn clip(s: &str) -> String {
    s.trim().chars().take(MAX_LINE_CHARS).collect()
}

fn parse_time(tag: &str) -> Option<Ms> {
    let (min, rest) = tag.split_once(':')?;
    if min.is_empty() || min.len() > 3 || !min.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (sec, frac) = match rest.find(['.', ':']) {
        Some(i) => (&rest[..i], &rest[i + 1..]),
        None => (rest, ""),
    };
    if sec.len() != 2 || !sec.bytes().all(|b| b.is_ascii_digit()) || frac.len() > 3 || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let sec: Ms = sec.parse().ok()?;
    if sec >= 60 {
        return None;
    }
    let frac_ms: Ms = match frac.len() {
        0 => 0,
        1 => frac.parse::<Ms>().ok()? * 100,
        2 => frac.parse::<Ms>().ok()? * 10,
        _ => frac.parse().ok()?,
    };
    Some(min.parse::<Ms>().ok()? * 60_000 + sec * 1000 + frac_ms)
}

impl Lyrics {
    pub fn parse_lrc(text: &str) -> Lyrics {
        let mut offset: i64 = 0;
        let mut lines = Vec::new();
        for raw in text.lines() {
            let mut rest = raw.trim();
            let mut stamps = Vec::new();
            while let Some(after) = rest.strip_prefix('[') {
                let Some(end) = after.find(']') else { break };
                let tag = &after[..end];
                match parse_time(tag) {
                    Some(t) => stamps.push(t),
                    None => {
                        if let Some(v) = tag.strip_prefix("offset:") {
                            offset = v.trim().parse().unwrap_or(0);
                        }
                    }
                }
                rest = after[end + 1..].trim_start();
            }
            for t in stamps {
                if lines.len() >= MAX_LINES {
                    break;
                }
                let at = (t as i64 - offset).max(0) as Ms;
                lines.push(Line { at, text: clip(rest) });
            }
        }
        lines.sort_by_key(|l| l.at);
        let synced = !lines.is_empty();
        Lyrics { lines, synced, instrumental: false }
    }

    pub fn plain(text: &str) -> Lyrics {
        let lines = text
            .lines()
            .map(clip)
            .filter(|l| !l.is_empty())
            .take(MAX_LINES)
            .map(|text| Line { at: 0, text })
            .collect();
        Lyrics { lines, synced: false, instrumental: false }
    }

    pub fn instrumental() -> Lyrics {
        Lyrics { lines: Vec::new(), synced: false, instrumental: true }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.text.is_empty())
    }

    pub fn index_at(&self, pos: Ms, duration: Ms) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        if self.synced {
            return self.lines.iter().rposition(|l| l.at <= pos);
        }
        if duration == 0 {
            return Some(0);
        }
        let n = self.lines.len() as u128;
        Some(((pos.min(duration) as u128 * n / duration as u128) as usize).min(self.lines.len() - 1))
    }

    pub fn next_change(&self, pos: Ms, duration: Ms) -> Option<Ms> {
        if self.synced {
            return self.lines.iter().find(|l| l.at > pos).map(|l| l.at);
        }
        let n = self.lines.len() as u64;
        if n < 2 || duration == 0 {
            return None;
        }
        let i = self.index_at(pos, duration)? as u64 + 1;
        (i < n).then(|| (i * duration).div_ceil(n))
    }

    pub fn line(&self, i: usize) -> Option<&str> {
        self.lines.get(i).map(|l| l.text.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.s.get(self.i).is_some_and(|b| b.is_ascii_whitespace()) {
            self.i += 1;
        }
    }

    fn eat(&mut self, lit: &str) -> Option<()> {
        self.s[self.i..].starts_with(lit.as_bytes()).then(|| self.i += lit.len())
    }

    fn hex4(&mut self) -> Option<u32> {
        let h = std::str::from_utf8(self.s.get(self.i..self.i + 4)?).ok()?;
        self.i += 4;
        u32::from_str_radix(h, 16).ok()
    }

    fn string(&mut self) -> Option<String> {
        self.eat("\"")?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let b = *self.s.get(self.i)?;
            self.i += 1;
            match b {
                b'"' => return String::from_utf8(out).ok(),
                b'\\' => {
                    let e = *self.s.get(self.i)?;
                    self.i += 1;
                    let c = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let hi = self.hex4()?;
                            let code = if (0xD800..0xDC00).contains(&hi) {
                                self.eat("\\u")?;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return None;
                                }
                                0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                            } else {
                                hi
                            };
                            char::from_u32(code)?
                        }
                        _ => return None,
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                }
                0..0x20 => return None,
                _ => out.push(b),
            }
        }
    }

    fn number(&mut self) -> Option<f64> {
        let start = self.i;
        while self.s.get(self.i).is_some_and(|b| b"+-.eE0123456789".contains(b)) {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok()?.parse().ok()
    }

    fn value(&mut self, depth: u32) -> Option<Json> {
        if depth > MAX_DEPTH {
            return None;
        }
        self.ws();
        let v = match *self.s.get(self.i)? {
            b'"' => Json::Str(self.string()?),
            b'{' => {
                self.i += 1;
                let mut fields = Vec::new();
                self.ws();
                if self.eat("}").is_none() {
                    loop {
                        self.ws();
                        let k = self.string()?;
                        self.ws();
                        self.eat(":")?;
                        let v = self.value(depth + 1)?;
                        fields.push((k, v));
                        self.ws();
                        if self.eat(",").is_some() {
                            continue;
                        }
                        self.eat("}")?;
                        break;
                    }
                }
                Json::Obj(fields)
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.eat("]").is_none() {
                    loop {
                        items.push(self.value(depth + 1)?);
                        self.ws();
                        if self.eat(",").is_some() {
                            continue;
                        }
                        self.eat("]")?;
                        break;
                    }
                }
                Json::Arr(items)
            }
            b't' => self.eat("true").map(|_| Json::Bool(true))?,
            b'f' => self.eat("false").map(|_| Json::Bool(false))?,
            b'n' => self.eat("null").map(|_| Json::Null)?,
            _ => Json::Num(self.number()?),
        };
        Some(v)
    }
}

pub fn parse_json(text: &str) -> Option<Json> {
    if text.len() > MAX_RESPONSE_BYTES {
        return None;
    }
    let mut p = Parser { s: text.as_bytes(), i: 0 };
    let v = p.value(0)?;
    p.ws();
    (p.i == p.s.len()).then_some(v)
}

fn from_record(r: &Json) -> Option<Lyrics> {
    if r.get("instrumental").and_then(Json::as_bool) == Some(true) {
        return Some(Lyrics::instrumental());
    }
    if let Some(l) = r.get("syncedLyrics").and_then(Json::as_str).map(Lyrics::parse_lrc).filter(|l| !l.is_empty()) {
        return Some(l);
    }
    r.get("plainLyrics").and_then(Json::as_str).map(Lyrics::plain).filter(|l| !l.is_empty())
}

pub fn pick(json: &Json, duration: Ms) -> Option<Lyrics> {
    match json {
        Json::Obj(_) => from_record(json),
        Json::Arr(items) => {
            let close = |r: &&Json| {
                let d = r.get("duration").and_then(Json::as_f64).map(|s| (s * 1000.0) as Ms);
                duration == 0 || d.is_none_or(|d| d.abs_diff(duration) <= DURATION_SLACK_MS)
            };
            let synced = items
                .iter()
                .filter(close)
                .find(|r| r.get("syncedLyrics").and_then(Json::as_str).is_some_and(|s| !s.trim().is_empty()));
            synced.or_else(|| items.iter().find(close)).and_then(from_record)
        }
        _ => None,
    }
}

pub fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

const NOISE: &[&str] = &[
    "official", "video", "lyric", "audio", "visualizer", "visualiser", "mv", "m/v", "hd", "4k", "remaster", "live",
    "explicit", "clean", "color coded", "sub", "eng", "romanized",
];

fn noisy(group: &str) -> bool {
    let g = group.to_ascii_lowercase();
    let word = |w: &str| {
        g.split(|c: char| !c.is_alphanumeric() && c != '/').any(|t| t == w || (w.len() >= 5 && t.starts_with(w)))
    };
    NOISE.iter().any(|w| if w.contains(' ') { g.contains(w) } else { word(w) })
}

pub fn clean_title(title: &str) -> String {
    let mut out = String::new();
    let mut rest = title;
    while let Some(i) = rest.find(['(', '[']) {
        let close = if rest.as_bytes()[i] == b'(' { ')' } else { ']' };
        out.push_str(&rest[..i]);
        match rest[i + 1..].find(close) {
            Some(j) => {
                let group = &rest[i + 1..i + 1 + j];
                if !noisy(group) {
                    out.push_str(&rest[i..i + j + 2]);
                }
                rest = &rest[i + j + 2..];
            }
            None => {
                out.push_str(&rest[i..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    let lower = out.to_ascii_lowercase();
    let cut = [" | ", " ft. ", " feat. "].iter().filter_map(|p| lower.find(p)).min();
    let out = match cut {
        Some(i) => &out[..i],
        None => &out,
    };
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn clean_artist(artist: &str) -> String {
    let mut a = artist.trim();
    for suffix in [" - Topic", "VEVO", "Official"] {
        if let Some(s) = a.strip_suffix(suffix) {
            a = s.trim_end();
        }
    }
    a.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Ms,
}

impl Query {
    pub fn new(title: &str, artist: &str, album: &str, duration: Ms) -> Query {
        let mut title = clean_title(title);
        let mut artist = clean_artist(artist);
        if let Some((a, t)) = title.split_once(" - ").map(|(a, t)| (a.trim().to_string(), t.trim().to_string()))
            && !t.is_empty()
            && (artist.is_empty() || a.eq_ignore_ascii_case(&artist) || album.trim().is_empty())
        {
            artist = a;
            title = t;
        }
        Query { title, artist, album: album.trim().to_string(), duration }
    }

    pub fn is_usable(&self) -> bool {
        !self.title.is_empty()
    }

    pub fn get_path(&self) -> String {
        let mut p = format!(
            "/api/get?track_name={}&artist_name={}",
            encode_component(&self.title),
            encode_component(&self.artist)
        );
        if !self.album.is_empty() {
            p.push_str(&format!("&album_name={}", encode_component(&self.album)));
        }
        if self.duration > 0 {
            p.push_str(&format!("&duration={}", self.duration.div_ceil(1000)));
        }
        p
    }

    pub fn search_path(&self) -> String {
        let mut p = format!("/api/search?track_name={}", encode_component(&self.title));
        if !self.artist.is_empty() {
            p.push_str(&format!("&artist_name={}", encode_component(&self.artist)));
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrc_dasar_dan_offset() {
        let l = Lyrics::parse_lrc("[ar:Notchi]\n[offset:+500]\n[00:01.00]Satu\n[00:03.5]Dua\n[01:02.345][00:02.00]Ulang\n[00:04.00]\n");
        assert!(l.synced);
        let at: Vec<Ms> = l.lines.iter().map(|l| l.at).collect();
        assert_eq!(at, vec![500, 1_500, 3_000, 3_500, 61_845]);
        assert_eq!(l.line(1), Some("Ulang"));
        assert_eq!(l.line(3), Some(""));
    }

    #[test]
    fn tag_rusak_diabaikan() {
        let l = Lyrics::parse_lrc("[xx:01.00]A\n[00:61.00]B\n[0:5]C\n[00:05]D\nE\n");
        assert_eq!(l.lines, vec![Line { at: 5_000, text: "D".into() }]);
    }

    #[test]
    fn indeks_dan_pergantian_tersinkron() {
        let l = Lyrics::parse_lrc("[00:01.00]A\n[00:03.00]B\n[00:07.00]C\n");
        assert_eq!(l.index_at(500, 10_000), None);
        assert_eq!(l.index_at(1_000, 10_000), Some(0));
        assert_eq!(l.index_at(6_999, 10_000), Some(1));
        assert_eq!(l.next_change(1_000, 10_000), Some(3_000));
        assert_eq!(l.next_change(7_000, 10_000), None);
    }

    #[test]
    fn lirik_polos_diperkirakan_dari_durasi() {
        let l = Lyrics::plain("a\n\nb\nc\nd\n");
        assert!(!l.synced);
        assert_eq!(l.lines.len(), 4);
        assert_eq!(l.index_at(0, 100_000), Some(0));
        assert_eq!(l.index_at(60_000, 100_000), Some(2));
        assert_eq!(l.index_at(200_000, 100_000), Some(3));
        assert_eq!(l.next_change(0, 100_000), Some(25_000));
    }

    #[test]
    fn json_lengkap() {
        let j = parse_json(r#" {"a": [1, -2.5e1, true, null], "s": "x\"y\\n\u00e9\ud83c\udfb5", "o": {}} "#).unwrap();
        assert_eq!(j.get("s").and_then(Json::as_str), Some("x\"y\\n\u{e9}\u{1F3B5}"));
        assert_eq!(j.get("a"), Some(&Json::Arr(vec![Json::Num(1.0), Json::Num(-25.0), Json::Bool(true), Json::Null])));
        assert_eq!(j.get("o"), Some(&Json::Obj(vec![])));
    }

    #[test]
    fn json_tidak_valid_ditolak() {
        for bad in ["", "{", "{\"a\":}", "[1,]", "\"\\ud800\"", "{\"a\":1} x", "\"a\nb\"", "tru"] {
            assert_eq!(parse_json(bad), None, "{bad:?}");
        }
        let deep = "[".repeat(40) + &"]".repeat(40);
        assert_eq!(parse_json(&deep), None);
    }

    #[test]
    fn pilih_dari_lrclib() {
        let get = parse_json(r#"{"id":1,"instrumental":false,"plainLyrics":"a\nb","syncedLyrics":"[00:01.00] a\n[00:02.00] b"}"#).unwrap();
        let l = pick(&get, 0).unwrap();
        assert!(l.synced);
        assert_eq!(l.lines.len(), 2);
        let search = parse_json(
            r#"[{"duration":300,"syncedLyrics":"[00:01.00]jauh"},{"duration":201,"plainLyrics":"polos"},{"duration":199.5,"syncedLyrics":"[00:01.00]dekat"}]"#,
        )
        .unwrap();
        assert_eq!(pick(&search, 200_000).unwrap().line(0), Some("dekat"));
        let inst = parse_json(r#"{"instrumental":true,"plainLyrics":null,"syncedLyrics":null}"#).unwrap();
        assert!(pick(&inst, 0).unwrap().instrumental);
        assert_eq!(pick(&parse_json(r#"{"code":404,"name":"TrackNotFound"}"#).unwrap(), 0), None);
    }

    #[test]
    fn url_dikodekan() {
        assert_eq!(encode_component("Mr. Brightside & Co/é"), "Mr.%20Brightside%20%26%20Co%2F%C3%A9");
    }

    #[test]
    fn judul_dibersihkan() {
        assert_eq!(clean_title("Hati-Hati di Jalan (Official Music Video)"), "Hati-Hati di Jalan");
        assert_eq!(clean_title("Song [Lyrics] (feat. Someone)"), "Song (feat. Someone)");
        assert_eq!(clean_title("Lagu ft. Teman | Live at Home"), "Lagu");
        assert_eq!(clean_title("Remix (Night Version)"), "Remix (Night Version)");
        assert_eq!(clean_artist("Tulus - Topic"), "Tulus");
        assert_eq!(clean_artist("TaylorSwiftVEVO"), "TaylorSwift");
    }

    #[test]
    fn query_dari_judul_video() {
        let q = Query::new("Tulus - Hati-Hati di Jalan (Official Video)", "TULUS", "", 242_400);
        assert_eq!((q.artist.as_str(), q.title.as_str()), ("Tulus", "Hati-Hati di Jalan"));
        assert_eq!(q.get_path(), "/api/get?track_name=Hati-Hati%20di%20Jalan&artist_name=Tulus&duration=243");
        let q = Query::new("Hati-Hati di Jalan", "Tulus", "Manusia", 0);
        assert!(q.get_path().ends_with("&album_name=Manusia"));
        assert_eq!(q.search_path(), "/api/search?track_name=Hati-Hati%20di%20Jalan&artist_name=Tulus");
    }
}
