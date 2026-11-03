# Notchi

Island interaktif, menu bar, dan Dock bergaya macOS untuk Windows 11, ditulis dengan Rust agar ringan.

> **Status:** Island Fase 1 sudah berjalan (Rust): perilaku Otomatis/Menempel/Sembunyi, jendela Pengaturan (`Notchi.exe --settings`), gaya gelap Neumorphism dan Clay 3D, maskot interaktif, partikel, serta lirik tersinkron opsional dari LRCLIB. Dock dan menu bar belum. Dokumen acuannya ada di [`docs/PRD.md`](docs/PRD.md).

## Unduh

Exe portabel tersedia di [GitHub Releases](https://github.com/devaldicaliesta/devnotch/releases/latest): unduh `Notchi-<versi>-windows-x64.exe`, lalu jalankan. Tidak perlu instalasi atau hak admin. Butuh Windows 11 22H2 atau lebih baru (x64).

Setiap tag `v*` memicu workflow [`release.yml`](.github/workflows/release.yml), yang menguji, membangun, lalu melampirkan exe, zip, dan `SHA256SUMS.txt` ke rilisnya.

## Rencana singkat

- **Island lebih dulu** (Fase 1–2): pil interaktif di tengah atas layar untuk musik/video (termasuk YouTube Music dan Apple Music), timer, pengisian daya, notifikasi, dan lainnya, dengan maskot orisinal yang juga bernama **Notchi**.
- **Dock dan menu bar** menyusul (Fase 3), lalu panel kontrol cepat dan launcher (Fase 4).
- Hanya **Windows 11 22H2 ke atas**.
- Dikemas sebagai **MSIX** (untuk notifikasi) dan sebagai `.exe` portabel.
- Target: ≤ 60 MB RAM dan ≤ 0,3% CPU saat idle, tanpa hak admin, tanpa telemetri. Terukur pada Island Fase 1: 0% CPU dan 1–3 MB private working set saat diam.

## Gratis dan sumber terbuka

- Kode: [MIT](LICENSE).
- Aset seni dan maskot: CC BY 4.0, lihat [`LICENSE-ASSETS.md`](LICENSE-ASSETS.md).
- Nama dan logo: lihat [`TRADEMARKS.md`](TRADEMARKS.md).
- Tanpa iklan, tanpa telemetri, tanpa fitur berbayar.

## Penafian

Notchi tidak berafiliasi dengan, disponsori, atau didukung oleh Apple, Microsoft, Google, atau OpenAI. Semua nama dan merek dagang adalah milik pemiliknya masing-masing dan hanya disebut secara deskriptif.

## Berkontribusi

Lihat [`CONTRIBUTING.md`](CONTRIBUTING.md). Laporan keamanan: [`SECURITY.md`](SECURITY.md).
