//! Format waktu untuk tampilan.

use crate::Ms;

/// `m:ss` atau `h:mm:ss`, dibulatkan ke atas agar hitung mundur tidak
/// menampilkan 0:00 sebelum benar-benar selesai.
pub fn countdown(remaining: Ms) -> String {
    clock(remaining.div_ceil(1000))
}

/// `m:ss` atau `h:mm:ss`, dibulatkan ke bawah (stopwatch, progres media).
pub fn elapsed(ms: Ms) -> String {
    clock(ms / 1000)
}

fn clock(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_hitung_mundur() {
        assert_eq!(countdown(0), "0:00");
        assert_eq!(countdown(1), "0:01");
        assert_eq!(countdown(59_001), "1:00");
        assert_eq!(countdown(300_000), "5:00");
        assert_eq!(countdown(3_600_000), "1:00:00");
    }

    #[test]
    fn format_berlalu() {
        assert_eq!(elapsed(999), "0:00");
        assert_eq!(elapsed(187_000), "3:07");
        assert_eq!(elapsed(3_725_000), "1:02:05");
    }
}
