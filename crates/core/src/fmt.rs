use crate::Ms;

pub fn countdown(remaining: Ms) -> String {
    clock(remaining.div_ceil(1000))
}

pub fn elapsed(ms: Ms) -> String {
    clock(ms / 1000)
}

/// Ukuran berkas ringkas: "820 KB", "12,4 MB", "1,2 GB". `decimal` adalah pemisah desimal bahasa.
pub fn bytes(n: u64, decimal: char) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if n < 1000 {
        return format!("{n} B");
    }
    let mut v = n as f64 / 1000.0;
    let mut i = 0;
    while v >= 999.5 && i < UNITS.len() - 1 {
        v /= 1000.0;
        i += 1;
    }
    let s = if v < 99.95 && i > 0 { format!("{v:.1}") } else { format!("{v:.0}") };
    format!("{} {}", s.replace('.', &decimal.to_string()), UNITS[i])
}

/// Kecepatan unduh: "3,1 MB/dtk" (Indonesia) atau "3.1 MB/s".
pub fn rate(bytes_per_sec: f64, decimal: char, per_sec: &str) -> String {
    format!("{}/{per_sec}", bytes(bytes_per_sec.max(0.0) as u64, decimal))
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
    fn format_ukuran() {
        assert_eq!(bytes(512, ','), "512 B");
        assert_eq!(bytes(820_000, ','), "820 KB");
        assert_eq!(bytes(12_400_000, ','), "12,4 MB");
        assert_eq!(bytes(1_200_000_000, '.'), "1.2 GB");
        assert_eq!(bytes(999_600, ','), "1,0 MB");
        assert_eq!(rate(3_100_000.0, ',', "dtk"), "3,1 MB/dtk");
    }

    #[test]
    fn format_berlalu() {
        assert_eq!(elapsed(999), "0:00");
        assert_eq!(elapsed(187_000), "3:07");
        assert_eq!(elapsed(3_725_000), "1:02:05");
    }
}
