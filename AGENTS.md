# Panduan untuk agen AI

Notchi adalah Island bergaya macOS untuk Windows 11, ditulis dengan Rust. Dokumen acuan produk ada di `docs/PRD.md`.

## Cari file lewat Graphify dulu

Repo ini punya peta kode dari [Graphify](https://pypi.org/project/graphifyy/) di `graphify-out/`:

- `graphify-out/GRAPH_REPORT.md`: ringkasan arsitektur, hub, dan komunitas modul. Baca ini sebelum menjelajah file.
- `graphify-out/graph.json`: graf simbol (fungsi, struct, enum) beserta file dan baris asalnya.

Pasang sekali, lalu tanya grafnya alih-alih grep buta:

```bash
pip install graphifyy
graphify query "palet tema terang dan kaca" --budget 1500
graphify explain "IslandApp"
graphify affected "Island"
```

Setiap node menyebut `src=<file> loc=L<baris>`, jadi langsung bisa dibuka.

## Wajib setelah mengubah kode

1. Perbarui graf (lokal, tanpa API, sekitar 2 detik), lalu commit `graphify-out/GRAPH_REPORT.md` dan `graphify-out/graph.json` bersama perubahan:

   ```bash
   graphify update .
   ```

2. Jalankan pemeriksaan (crate `platform-win` dan `island` hanya untuk Windows; di Linux pakai target silang):

   ```bash
   cargo test -p notchi-core
   cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings
   ```

3. Buka PR ke `main`. Setelah digabung, rilis: naikkan `version` di `Cargo.toml`, tulis `docs/rilis/vX.Y.Z.md`, lalu jalankan workflow **Release** (`workflow_dispatch` dengan input `tag = vX.Y.Z`, atau push tag `vX.Y.Z`). Tag harus sama dengan versi Cargo.

## Peta singkat

| Crate | Isi |
| --- | --- |
| `crates/core` | Logika murni yang bisa diuji di semua OS: state Island (`island.rs`), tata letak (`layout.rs`), konfigurasi (`config.rs`), tema/palet (`theme.rs`), teks (`i18n.rs`), media, timer, lirik, pelacak unduhan (`downloads.rs`), pelacak Bluetooth (`bluetooth.rs`), rak berkas (`shelf.rs`). |
| `crates/platform-win` | Pembungkus Win32/WinRT: jendela, komposisi, Direct2D (`gfx.rs`), media, registry dan folder (`shell.rs`), `DeviceWatcher` Bluetooth (`bluetooth.rs`), kecerahan WMI/DDC-CI (`brightness.rs`), seret-lepas OLE (`dragdrop.rs`). |
| `crates/island` | Aplikasi Island: `lib.rs` (event, render, drag, region), `views.rs` (isi pil/kartu), `ui.rs` (primitif gambar, pil dan blur acrylic), `settings.rs` (jendela Pengaturan). |
| `crates/app` | Titik masuk `Notchi.exe`. |

Bahasa kode, komentar, dan commit: Indonesia.
