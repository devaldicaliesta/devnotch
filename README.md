# Notchi

Island interaktif, menu bar, dan Dock bergaya macOS untuk Windows 11, ditulis dengan Rust agar ringan.

> **Status:** Island Fase 1 sudah berjalan (Rust): perilaku Otomatis/Menempel/Sembunyi, jendela Pengaturan (`Notchi.exe --settings`), gaya Neumorphism, Clay 3D, Kaca (glass), dan Klasik dengan tema terang/gelap (**Pengaturan → Tema**, default mengikuti Windows), maskot interaktif, partikel, serta lirik tersinkron opsional dari LRCLIB. Sejak v0.2.0: peek interaktif saat notch di-hover (jam, tanggal, baterai, maskot; atau judul lagu dan putar/jeda), Island menyingkir dari tab browser, seek di bilah progres, klik tengah untuk putar/jeda, scroll untuk mengatur timer, dan indikator mikrofon/kamera. Sejak v0.2.1: Island bisa dipindah ke kiri atau kanan atas (**Pengaturan → Posisi**, atau seret pil ke kiri/kanan), dan zona hover-nya kecil secara default (**Pengaturan → Zona hover**) agar tidak muncul saat memilih tab browser. Saat di samping, maskot kecil bisa ikut menemani di tepi atas (**Pengaturan → Maskot di samping**) dengan mata yang mengikuti kursor. Sejak v0.5.0: pil unduhan browser dengan alert selesai, alert perangkat Bluetooth (dengan baterai bila dilaporkan), dan Shift + scroll untuk kecerahan layar. Sejak v0.6.0: rak berkas (seret berkas ke Island, lalu seret keluar ke aplikasi lain) dan blur acrylic untuk gaya Kaca. Dock dan menu bar belum. Dokumen acuannya ada di [`docs/PRD.md`](docs/PRD.md).

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

- Kode: [Apache 2.0](LICENSE) (lihat juga [NOTICE](NOTICE)). Rilis hingga v0.5.0 berlisensi MIT.
- Aset seni dan maskot: CC BY 4.0, lihat [`LICENSE-ASSETS.md`](LICENSE-ASSETS.md).
- Nama dan logo: lihat [`TRADEMARKS.md`](TRADEMARKS.md).
- Tanpa iklan, tanpa telemetri, tanpa fitur berbayar.

## Penafian

Notchi tidak berafiliasi dengan, disponsori, atau didukung oleh Apple, Microsoft, Google, atau OpenAI. Semua nama dan merek dagang adalah milik pemiliknya masing-masing dan hanya disebut secara deskriptif.

## Berkontribusi

Lihat [`CONTRIBUTING.md`](CONTRIBUTING.md). Laporan keamanan: [`SECURITY.md`](SECURITY.md).
