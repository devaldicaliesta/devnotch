# Notchi

Island interaktif bergaya macOS untuk Windows 11: pil kecil di tepi atas layar yang menampilkan musik, timer, unduhan, perangkat Bluetooth, dan lainnya, ditemani maskot bernama **Notchi**. Ditulis dengan Rust, ringan (sekitar 0% CPU dan beberapa MB RAM saat diam), tanpa telemetri.

[**⬇ Unduh versi terbaru**](https://github.com/devaldicaliesta/devnotch/releases/latest) · [Catatan rilis](docs/rilis) · [Dukung pengembangan](https://yapp.ink/ImDev)

## Fitur

- **Media**: judul, artis, sampul, progres yang bisa di-seek, dan tombol putar/jeda untuk Spotify, YouTube Music, Apple Music, atau pemutar apa pun yang memakai kontrol media Windows. Ada lirik tersinkron opsional (LRCLIB).
- **Timer dan stopwatch**: preset cepat; scroll di atas pil untuk menambah atau mengurangi menit.
- **Unduhan**: progres unduhan browser, dan alert saat selesai yang membuka berkasnya di Explorer.
- **Rak berkas**: seret berkas ke Island untuk disimpan sementara, lalu seret keluar ke folder, WhatsApp, email, atau aplikasi lain.
- **Alert sistem**: pengisian daya, baterai rendah, perangkat Bluetooth tersambung/terputus (dengan baterai), dan indikator mikrofon/kamera.
- **Volume dan kecerahan**: scroll di atas pil untuk volume, Shift + scroll untuk kecerahan layar.
- **Tampilan**: gaya Neumorphism, Clay 3D, Kaca (blur acrylic), atau Klasik, dengan tema terang/gelap yang mengikuti Windows. Island bisa di tengah, kiri, atau kanan, dan bisa diseret.
- **Maskot**: ekspresinya mengikuti aktivitas, matanya mengikuti kursor, dan bisa di-boop.

## Mulai

1. Unduh `Notchi-<versi>-windows-x64.exe` dari [Releases](https://github.com/devaldicaliesta/devnotch/releases/latest) lalu jalankan. Tidak perlu instalasi atau hak admin.
2. Butuh Windows 11 22H2 atau lebih baru (x64). Exe belum bertanda tangan digital; bila SmartScreen muncul, pilih **Info selengkapnya → Tetap jalankan**.

| Aksi | Hasil |
| --- | --- |
| Hover | Jam, tanggal, dan baterai; atau lagu yang diputar |
| Klik / tahan | Buka atau tutup kartu |
| Klik kanan | Menu: timer, perilaku, sembunyikan 1 jam, **Pengaturan** |
| Seret pil ke kiri/kanan | Pindahkan Island di sepanjang tepi atas |
| `Ctrl+Alt+N` | Buka/tutup tanpa mouse (bisa diubah) |

**Perilaku** di Pengaturan: *Otomatis* (masuk ke tepi dan muncul saat kursor ditahan di tepi atas), *Menempel* (selalu tampil), atau *Sembunyi* (hanya saat ada aktivitas).

Semua pengaturan tersimpan di `%APPDATA%\Notchi\config.toml` dan dimuat ulang otomatis saat berkas disimpan. Log ada di `%LOCALAPPDATA%\Notchi\notchi.log`.

## Membangun dari sumber

```powershell
cargo build --release -p notchi
```

Toolchain Rust dipasang otomatis dari `rust-toolchain.toml`. Hasilnya ada di `target\release\Notchi.exe`. Arsitektur dan rencana produk ada di [`docs/PRD.md`](docs/PRD.md), panduan untuk kontributor dan agen AI di [`CONTRIBUTING.md`](CONTRIBUTING.md) dan [`AGENTS.md`](AGENTS.md).

## Rencana

Island sudah berjalan. Berikutnya: notifikasi aplikasi (paket MSIX), lalu Dock dan menu bar, panel kontrol cepat, dan launcher.

## Lisensi

- Kode: [Apache 2.0](LICENSE) (lihat [NOTICE](NOTICE)). Rilis hingga v0.5.0 berlisensi MIT.
- Maskot dan aset seni: CC BY 4.0 ([`LICENSE-ASSETS.md`](LICENSE-ASSETS.md)). Nama dan logo: [`TRADEMARKS.md`](TRADEMARKS.md).
- Gratis, tanpa iklan, tanpa telemetri, tanpa fitur berbayar. Bila bermanfaat, dukung lewat [yapp.ink/ImDev](https://yapp.ink/ImDev).

Notchi tidak berafiliasi dengan Apple, Microsoft, Google, atau OpenAI; nama dan merek dagang disebut secara deskriptif. Laporan keamanan: [`SECURITY.md`](SECURITY.md).
