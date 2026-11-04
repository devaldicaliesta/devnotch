# Graph Report - devnotch  (2026-11-04)

## Corpus Check
- 68 files · ~65,492 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 5 file(s) not represented in the graph (top: (none) 4, .toml 1)

## Summary
- 2410 nodes · 4738 edges · 131 communities (97 shown, 34 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 169 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `52cdcca1`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- window.rs
- .handle
- views.rs
- Text
- gfx.rs
- island.rs
- config.rs
- island/src/lib.rs
- brightness.rs
- Ms
- ActivityKind
- layout.rs
- core/src/media.rs
- Event
- Ui
- Window
- net.rs
- Settings
- shell.rs
- settings.rs
- release.yml
- Setting
- HWND
- steps
- Gfx
- platform-win/src/media.rs
- Point
- lyrics.rs
- State
- HWND
- Color
- Shape
- TrayIcon
- Model
- Json
- .layout_window
- Placement
- platform-win/src/privacy.rs
- Animator
- ViewKind
- TextStyle
- DeviceKind
- Size
- Button
- result
- main.rs
- Lyrics
- README.md
- Lang
- dragdrop.rs
- fswatch.rs
- HKEY
- Shelf
- Style
- PowerReading
- downloads.rs
- .show_menu
- .new
- 9. Rilis, pengujian, peta jalan, dan risiko
- v0.2.0.md
- fmt.rs
- 2. Spesifikasi visual dan catatan legal
- 3. Fitur Island (pil interaktif di tengah atas layar)
- v0.3.0.md
- SpringPreset
- PRD — Notchi: Island dan Dock untuk Windows 11 (Rust)
- 5. Menu bar, panel kontrol, launcher, dan pengaturan
- 6. Arsitektur teknis (Rust)
- 7. Keamanan dan privasi
- v0.1.0.md
- v0.2.1.md
- Notchi
- cli
- platform-win/src/bluetooth.rs
- open_run_key
- init_process
- 4. Fitur Dock (pengganti taskbar bawah, Fase 3)
- smoke.ps1
- Artwork
- LyricStatus
- InstanceLock
- notchi
- PCWSTR
- Island
- theme.rs
- comp.rs
- ui.rs
- v0.4.0.md
- Panduan untuk agen AI
- AlertKind
- Palette
- Config
- Behavior
- semua_elemen_kartu_di_dalam_kartu
- Skin
- icon.rs
- Batch
- Presentation
- log
- Style
- Kind
- wmi_step
- UiProxy
- Priority
- Card
- LyricState
- sembunyi_otomatis_ke_tepi_atas
- ddc_step
- v0.5.0.md
- PrivacyDot
- Backend
- v0.6.0.md
- parse_ovh
- .new
- int_variant
- file_icon
- v0.6.1.md
- IslandApp

## God Nodes (most connected - your core abstractions)
1. `IslandApp` - 132 edges
2. `Ms` - 112 edges
3. `Text` - 103 edges
4. `Ui` - 65 edges
5. `Event` - 63 edges
6. `Rect` - 57 edges
7. `Size` - 44 edges
8. `Settings` - 40 edges
9. `Window` - 35 edges
10. `Island` - 35 edges

## Surprising Connections (you probably didn't know these)
- `all()` --references--> `Item`  [INFERRED]
  crates/core/src/theme.rs → crates/core/src/shelf.rs
- `cli()` --calls--> `reset_config()`  [INFERRED]
  crates/app/src/main.rs → crates/island/src/lib.rs
- `wndproc()` --calls--> `scale_for_dpi()`  [INFERRED]
  crates/platform-win/src/window.rs → crates/core/src/geom.rs
- `create_panel()` --calls--> `scale_for_dpi()`  [INFERRED]
  crates/platform-win/src/window.rs → crates/core/src/geom.rs
- `main()` --calls--> `run_message_loop()`  [INFERRED]
  crates/app/src/main.rs → crates/platform-win/src/window.rs

## Import Cycles
- 1-file cycle: `crates/core/src/activity.rs -> crates/core/src/activity.rs`

## Communities (131 total, 34 thin omitted)

### Community 0 - "window.rs"
Cohesion: 0.01
Nodes (15): CLASS_NAME, MAX_Z_WALK, PANEL_CLASS, PANEL_EX, PANEL_MIN_DP, PANEL_STYLE, quit(), WM_APP_APPBAR (+7 more)

### Community 1 - ".handle"
Cohesion: 0.13
Nodes (4): load_config(), Paths, resolve_mode(), now_ms()

### Community 2 - "views.rs"
Cohesion: 0.05
Nodes (40): build_bubble(), compact_size(), compact_width(), G_BLUETOOTH, G_BRIGHTNESS, G_CANCEL, G_CHEVRON, G_CLOCK (+32 more)

### Community 3 - "Text"
Cohesion: 0.02
Nodes (102): Text, AboutBody, Advanced, AdvancedDesc, AlreadyRunning, AvoidWindows, BatteryCritical, BatteryLow (+94 more)

### Community 5 - "island.rs"
Cohesion: 0.22
Nodes (14): alert_menyela_lalu_kembali_ke_state_semula(), alert_prioritas_rendah_tidak_menimpa(), charging(), hide_muncul_hanya_saat_aktivitas(), hud_volume_tidak_menutup_kartu(), jumlah_aktivitas_menentukan_bentuk(), klik_alert_membuka_kartu(), klik_expand_dan_collapse() (+6 more)

### Community 6 - "config.rs"
Cohesion: 0.06
Nodes (44): bagian_bukan_tabel(), berkas_kosong_memakai_default(), berkas_terlalu_besar_ditolak(), check_unknown(), daftar_dibatasi(), DEFAULT_HOTKEY, DEFAULT_TOML, finish() (+36 more)

### Community 7 - "island/src/lib.rs"
Cohesion: 0.03
Nodes (53): BRIGHTNESS_STEP, COLLAPSE_DELAY_MS, CONTENT_DELAY_MS, CONTENT_FADE_MS, COVER_SETTLE_MS, DOT, DOT_INSET, DOWNLOAD_DEBOUNCE_MS (+45 more)

### Community 9 - "Ms"
Cohesion: 0.10
Nodes (20): Ms, ContentHold, batas_durasi(), Countdown, CountdownState, Paused, Running, hitung_mundur_dasar() (+12 more)

### Community 10 - "ActivityKind"
Cohesion: 0.08
Nodes (32): Activities, ActivityKind, Download, Media, Privacy, Shelf, Stopwatch, Timer (+24 more)

### Community 11 - "layout.rs"
Cohesion: 0.06
Nodes (38): anchor_px(), BUBBLE_GAP, BUDDY_MASCOT, COMPACT_MAX_W, COMPACT_MIN_W, corner_radius(), EXPANDED_MAX_H, EXPANDED_MIN_H (+30 more)

### Community 12 - "core/src/media.rs"
Cohesion: 0.08
Nodes (26): art_fingerprint(), Controls, laporan_baru_dan_lagu_baru_memakai_timeline_segar(), laporan_posisi_rutin_dibuang(), media_dijeda_bertahan_sebentar(), MediaPresence, MediaSnapshot, nama_sumber() (+18 more)

### Community 13 - "Event"
Cohesion: 0.05
Nodes (46): Event, Bluetooth, BluetoothReady, BluetoothRemoved, Brightness, Close, DisplayChanged, DownloadsChanged (+38 more)

### Community 14 - "Ui"
Cohesion: 0.13
Nodes (15): Rect, Align, Center, Left, Right, DynText, Mascot, mascot() (+7 more)

### Community 15 - "Window"
Cohesion: 0.08
Nodes (4): Handler, OwnedWindow, Window, WindowState

### Community 16 - "net.rs"
Cohesion: 0.07
Nodes (6): Handle, https_get(), LYRICS_FALLBACK_HOST, LYRICS_HOST, Response, user_agent()

### Community 18 - "shell.rs"
Cohesion: 0.06
Nodes (4): local_time(), LocalTime, open_folder(), reveal()

### Community 19 - "settings.rs"
Cohesion: 0.08
Nodes (20): Burst, Confetti, Pop, Sparkle, Spark, Action, CARD_R, COL (+12 more)

### Community 20 - "release.yml"
Cohesion: 0.07
Nodes (29): concurrency, cancel-in-progress, group, run, CARGO_TERM_COLOR, GH_TOKEN, TAG, tag (+21 more)

### Community 21 - "Setting"
Cohesion: 0.08
Nodes (25): Setting, Advanced, AvoidWindows, Behavior, Bluetooth, Boop, Downloads, Fullscreen (+17 more)

### Community 22 - "HWND"
Cohesion: 0.14
Nodes (13): main(), arrange_event_proc(), create_overlay(), error_box(), fill_menu(), MenuItem, Item, Separator (+5 more)

### Community 23 - "steps"
Cohesion: 0.07
Nodes (28): concurrency, cancel-in-progress, group, run, CARGO_TERM_COLOR, jobs, windows-x64, name (+20 more)

### Community 24 - "Gfx"
Cohesion: 0.13
Nodes (3): d2d_color(), Gfx, Surface

### Community 25 - "platform-win/src/media.rs"
Cohesion: 0.06
Nodes (25): SourceFilter, MAX_ART_BYTES, COMMAND_SETTLE_MS, HOLD_MS, MediaCommand, CurrentChanged, CycleSession, Next (+17 more)

### Community 27 - "lyrics.rs"
Cohesion: 0.15
Nodes (18): bare_title(), clean_artist(), clean_title(), clip(), DURATION_SLACK_MS, indeks_dan_pergantian_tersinkron(), judul_dibersihkan(), lrc_dasar_dan_offset() (+10 more)

### Community 29 - "State"
Cohesion: 0.21
Nodes (5): aumid(), play_pause(), State, status_of(), Subscribed

### Community 30 - "HWND"
Cohesion: 0.19
Nodes (8): scale_for_dpi(), skala_dpi(), colorref(), frame_bounds(), is_app_window(), is_desktop(), mouse_point(), set_caption()

### Community 31 - "Color"
Cohesion: 0.14
Nodes (14): AA, accent_alert(), accent_charging(), Color, ISLAND_BG, kontras_hitam_putih_21(), semua_token_lulus_wcag_aa_di_atas_hitam(), STATUS_CRITICAL (+6 more)

### Community 32 - "Shape"
Cohesion: 0.28
Nodes (14): bibir_dan_zona_dekat(), compact_size(), hit_box(), minimal_terpusat_dengan_gelembung(), near_zone(), pil_merapat_ke_tepi_tanpa_terpotong(), pill_center(), region_diam_pas_dengan_pil() (+6 more)

### Community 33 - "TrayIcon"
Cohesion: 0.26
Nodes (4): create_hicon(), create_panel(), IconImage, TrayIcon

### Community 34 - "Model"
Cohesion: 0.23
Nodes (25): t(), ShelfItem, battery_badge(), build_alert(), build_buddy(), build_compact(), build_download_card(), build_home_card() (+17 more)

### Community 35 - "Json"
Cohesion: 0.20
Nodes (8): Json, Arr, Bool, Null, Num, Obj, Str, Parser

### Community 36 - ".layout_window"
Cohesion: 0.32
Nodes (3): cursor_pos(), Monitor, primary_monitor()

### Community 37 - "Placement"
Cohesion: 0.12
Nodes (9): HoverZone, .ALL, Normal, Small, Placement, .ALL, Center, Left (+1 more)

### Community 40 - "ViewKind"
Cohesion: 0.16
Nodes (14): PeekKind, Idle, Media, slot_of(), ViewKind, Alert, Buddy, Card (+6 more)

### Community 41 - "TextStyle"
Cohesion: 0.14
Nodes (6): TextStyle, Weight, Bold, Medium, Regular, Semibold

### Community 42 - "DeviceKind"
Cohesion: 0.16
Nodes (18): Bluetooth, Change, Device, DeviceKind, Audio, Controller, Keyboard, Mouse (+10 more)

### Community 43 - "Size"
Cohesion: 0.16
Nodes (13): Size, BATTERY_BADGE, BUBBLE, BUDDY, compact_slots(), HUD, IDLE, inside() (+5 more)

### Community 44 - "Button"
Cohesion: 0.11
Nodes (19): Button, Lyrics, Mascot, Next, OpenDownloads, PlayPause, Preset, Prev (+11 more)

### Community 47 - "Lyrics"
Cohesion: 0.17
Nodes (13): encode_component(), from_record(), Line, Lyrics, pick(), pilih_dari_lrclib(), Query, url_dikodekan() (+5 more)

### Community 48 - "README.md"
Cohesion: 0.19
Nodes (7): Aturan dasar, Berkontribusi, Lisensi aset, Cakupan, Kebijakan keamanan, Melaporkan kerentanan, Kebijakan nama dan logo

### Community 49 - "Lang"
Cohesion: 0.17
Nodes (14): bahasa(), DAYS_EN, DAYS_ID, decimal(), Lang, En, Id, long_date() (+6 more)

### Community 50 - "dragdrop.rs"
Cohesion: 0.07
Nodes (10): accept(), data_object(), drag_out(), DRAGGING_OUT, files_of(), has_files(), hdrop_format(), Pidls (+2 more)

### Community 51 - "fswatch.rs"
Cohesion: 0.12
Nodes (3): register(), watch_dir(), UiProxy

### Community 52 - "HKEY"
Cohesion: 0.24
Nodes (9): Privacy, any_in_use(), capability(), Key, open(), qword(), read(), subkeys() (+1 more)

### Community 53 - "Shelf"
Cohesion: 0.17
Nodes (8): buang_yang_hilang(), dibatasi_dan_yang_lama_dibuang(), Item, MAX_ITEMS, pb(), same(), Shelf, tambah_tanpa_duplikat()

### Community 54 - "Style"
Cohesion: 0.17
Nodes (8): Mode, Dark, Light, Theme, .ALL, Dark, Light, System

### Community 55 - "PowerReading"
Cohesion: 0.15
Nodes (14): ambang_sekali_per_siklus(), ambang_tidak_valid_dibuang(), bat(), BatteryMonitor, colok_dan_cabut(), mulai_dalam_kondisi_rendah(), pc_tanpa_baterai_diam(), PowerEvent (+6 more)

### Community 56 - "downloads.rs"
Cohesion: 0.21
Nodes (16): chrome_selesai_saat_berganti_nama(), dibatalkan_tanpa_alert(), Download, Downloads, final_name(), Finished, firefox_placeholder_tidak_dihitung_selesai_lebih_awal(), is_partial() (+8 more)

### Community 59 - "9. Rilis, pengujian, peta jalan, dan risiko"
Cohesion: 0.25
Nodes (8): 9.1 Peta jalan, 9.2 Pengemasan dan distribusi, 9.3 Strategi pengujian, 9.4 Risiko, 9.5 Definisi selesai untuk versi 1.0, 9.6 Pertanyaan terbuka, 9.7 Lisensi dan tata kelola open source, 9. Rilis, pengujian, peta jalan, dan risiko

### Community 60 - "v0.2.0.md"
Cohesion: 0.25
Nodes (7): Baru, Catatan, Island tidak lagi menghalangi tab browser, Pemutaran media, Perbaikan, Persyaratan, Unduhan

### Community 61 - "fmt.rs"
Cohesion: 0.36
Nodes (7): clock(), countdown(), elapsed(), format_berlalu(), format_hitung_mundur(), format_ukuran(), rate()

### Community 62 - "2. Spesifikasi visual dan catatan legal"
Cohesion: 0.29
Nodes (7): 2.1 Material kaca, 2.2 Bentuk, tipografi, ikon, 2.3 Gerak (motion), 2.4 Aksesibilitas, 2.5 Catatan legal dan kekayaan intelektual, 2.6 Maskot dan palet warna, 2. Spesifikasi visual dan catatan legal

### Community 63 - "3. Fitur Island (pil interaktif di tengah atas layar)"
Cohesion: 0.29
Nodes (7): 3.1 State dan ukuran, 3.2 Aktivitas live, 3.3 Interaksi, 3.4 Prioritas dan antrean, 3.5 Kriteria penerimaan Island, 3.6 Sumber media: YouTube Music, Apple Music, dan lainnya, 3. Fitur Island (pil interaktif di tengah atas layar)

### Community 64 - "v0.3.0.md"
Cohesion: 0.29
Nodes (6): Baru, Catatan, Perbaikan, Persyaratan, Perubahan perilaku, Unduhan

### Community 65 - "SpringPreset"
Cohesion: 0.16
Nodes (10): bouncy_memantul_sekali(), rumus_sesuai_prd(), semua_preset_selesai_dalam_400_ms(), smooth_tidak_melampaui_target(), Spring, SpringPreset, .BOUNCY, .SMOOTH (+2 more)

### Community 66 - "PRD — Notchi: Island dan Dock untuk Windows 11 (Rust)"
Cohesion: 0.33
Nodes (6): 1. Ringkasan, tujuan, dan metrik, 8.1 Anggaran sumber daya, 8.2 Kompatibilitas dan kasus tepi, 8.3 Cara mengukur, 8. Performa dan kompatibilitas, PRD — Notchi: Island dan Dock untuk Windows 11 (Rust)

### Community 67 - "5. Menu bar, panel kontrol, launcher, dan pengaturan"
Cohesion: 0.33
Nodes (6): 5.1 Menu bar, 5.2 Panel kontrol cepat, 5.3 Launcher ala Spotlight, 5.4 Pengaturan, 5.5 Kriteria penerimaan, 5. Menu bar, panel kontrol, launcher, dan pengaturan

### Community 68 - "6. Arsitektur teknis (Rust)"
Cohesion: 0.33
Nodes (6): 6.1 Keputusan arsitektur, 6.2 Struktur workspace, 6.3 Model thread, 6.4 Aturan render hemat, 6.5 Diagram arsitektur, 6. Arsitektur teknis (Rust)

### Community 69 - "7. Keamanan dan privasi"
Cohesion: 0.33
Nodes (6): 7.1 Model ancaman, 7.2 Pengerasan proses, 7.3 Pembaruan dan distribusi aman, 7.4 Privasi, 7.5 Pengujian dan kriteria penerimaan keamanan, 7. Keamanan dan privasi

### Community 70 - "v0.1.0.md"
Cohesion: 0.33
Nodes (5): Cara pakai, Catatan, Isi rilis, Persyaratan, Unduhan

### Community 71 - "v0.2.1.md"
Cohesion: 0.33
Nodes (5): Baru, Catatan, Perbaikan, Persyaratan, Unduhan

### Community 72 - "Notchi"
Cohesion: 0.33
Nodes (6): Berkontribusi, Gratis dan sumber terbuka, Notchi, Penafian, Rencana singkat, Unduh

### Community 73 - "cli"
Cohesion: 0.40
Nodes (4): cli(), config_path(), reset_config(), attach_parent_console()

### Community 74 - "platform-win/src/bluetooth.rs"
Cohesion: 0.10
Nodes (6): BATTERY, BluetoothWatcher, CLASSIC, CONNECTED, LE, props_of()

### Community 75 - "open_run_key"
Cohesion: 0.40
Nodes (3): launch_at_login_enabled(), open_run_key(), set_launch_at_login()

### Community 76 - "init_process"
Cohesion: 0.40
Nodes (3): Hardening, init_process(), set_policy()

### Community 77 - "4. Fitur Dock (pengganti taskbar bawah, Fase 3)"
Cohesion: 0.40
Nodes (5): 4.1 Strategi terhadap taskbar Windows, 4.2 Fitur, 4.3 Batasan yang harus diterima, 4.4 Kriteria penerimaan Dock, 4. Fitur Dock (pengganti taskbar bawah, Fase 3)

### Community 78 - "smoke.ps1"
Cohesion: 0.60
Nodes (3): Away(), Px(), Shot()

### Community 80 - "LyricStatus"
Cohesion: 0.22
Nodes (8): lyric_key(), LyricSlot, LyricStatus, Loading, Missing, Offline, Ready, query_of()

### Community 82 - "notchi"
Cohesion: 1.00
Nodes (4): notchi, notchi-core, notchi-island, notchi-platform-win

### Community 84 - "PCWSTR"
Cohesion: 0.33
Nodes (5): apps_use_light_theme(), personalize_dword(), RUN_KEY, RUN_VALUE, transparency_enabled()

### Community 86 - "theme.rs"
Cohesion: 0.16
Nodes (13): aksen_tetap_terbaca_di_tema_terang(), all(), BLACK, INK, kaca_tembus_pandang(), klasik_datar_tanpa_bayangan(), NONE, on_color() (+5 more)

### Community 87 - "comp.rs"
Cohesion: 0.09
Nodes (4): boxed_f32(), CompositionHost, LOOP_BUDGET_MS, REDUCED_MS

### Community 88 - "ui.rs"
Cohesion: 0.08
Nodes (13): burst(), EASE_OUT, GRAVITY, Rng, SOFT, sparkle_colors(), BLUSH, EYE (+5 more)

### Community 89 - "v0.4.0.md"
Cohesion: 0.33
Nodes (5): Baru, Catatan, Perbaikan, Persyaratan, Unduhan

### Community 90 - "Panduan untuk agen AI"
Cohesion: 0.40
Nodes (4): Cari file lewat Graphify dulu, Panduan untuk agen AI, Peta singkat, Wajib setelah mengubah kode

### Community 91 - "AlertKind"
Cohesion: 0.15
Nodes (11): ActiveAlert, AlertKind, Bluetooth, Brightness, Charging, DownloadDone, LowBattery, TimerDone (+3 more)

### Community 92 - "Palette"
Cohesion: 0.31
Nodes (4): Palette, Relief, shade(), tint()

### Community 93 - "Config"
Cohesion: 0.18
Nodes (8): BluetoothConfig, Config, DownloadsConfig, MediaConfig, ShelfConfig, behavior_desc(), Look, value()

### Community 94 - "Behavior"
Cohesion: 0.25
Nodes (5): Behavior, .ALL, Autohide, Hide, Sticky

### Community 95 - "semua_elemen_kartu_di_dalam_kartu"
Cohesion: 0.20
Nodes (11): alert_card(), AlertCard, download_card(), DownloadCard, semua_elemen_kartu_di_dalam_kartu(), shelf_card(), ShelfCard, ShelfTile (+3 more)

### Community 96 - "Skin"
Cohesion: 0.20
Nodes (9): Skin, .ALL, Karang, Langit, Lavender, Lemon, Mint, Permen (+1 more)

### Community 97 - "icon.rs"
Cohesion: 0.42
Nodes (8): ellipse_inside(), EYE, mascot_rgba(), mata_gelap(), px(), rounded_rect_sdf(), sudut_transparan_tengah_opak(), ukuran_buffer_benar()

### Community 101 - "Presentation"
Cohesion: 0.29
Nodes (7): Presentation, Alert, Compact, Expanded, Hidden, Idle, Minimal

### Community 103 - "Style"
Cohesion: 0.25
Nodes (6): Style, .ALL, Classic, Clay, Glass, Neumorphism

### Community 104 - "Kind"
Cohesion: 0.22
Nodes (7): Control, Kind, Button, Choice, Mascot, Swatch, Toggle

### Community 105 - "wmi_step"
Cohesion: 0.52
Nodes (4): prop(), wmi_first(), wmi_get(), wmi_step()

### Community 107 - "Priority"
Cohesion: 0.33
Nodes (6): Priority, P0, P1, P2, P3, P4

### Community 108 - "Card"
Cohesion: 0.25
Nodes (8): Card, Download, Home, Lyrics, Media, Shelf, Stopwatch, Timer

### Community 109 - "LyricState"
Cohesion: 0.40
Nodes (5): LyricState, Loading, Missing, Offline, Ready

### Community 111 - "ddc_step"
Cohesion: 0.40
Nodes (4): apply(), ddc_read(), ddc_step(), with_monitors()

### Community 112 - "v0.5.0.md"
Cohesion: 0.50
Nodes (3): Baru, Catatan, Unduhan

### Community 115 - "Backend"
Cohesion: 0.40
Nodes (6): Backend, Ddc, None, Wmi, open(), wmi_open()

### Community 116 - "v0.6.0.md"
Cohesion: 0.40
Nodes (4): Baru, Catatan, Lisensi, Unduhan

### Community 117 - "parse_ovh"
Cohesion: 0.29
Nodes (6): cadangan_lyrics_ovh(), json_lengkap(), json_tidak_valid_ditolak(), lirik_polos_diperkirakan_dari_durasi(), parse_json(), parse_ovh()

### Community 121 - "v0.6.1.md"
Cohesion: 0.50
Nodes (3): Perbaikan, Perubahan, Unduhan

### Community 127 - "IslandApp"
Cohesion: 0.17
Nodes (5): Content, IslandApp, lyric_timing(), rect_union(), ViewKey

## Knowledge Gaps
- **309 isolated node(s):** `Idle`, `Media`, `Hidden`, `Empty`, `Buddy` (+304 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1206 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **34 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Text` connect `Text` to `Lang`, `Model`?**
  _High betweenness centrality (0.107) - this node is a cross-community bridge._
- **What connects `Idle`, `Media`, `Hidden` to the rest of the system?**
  _309 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `window.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.010049751243781095 - nodes in this community are weakly interconnected._
- **Why does `IslandApp` connect `IslandApp` to `.handle`, `config.rs`, `island/src/lib.rs`, `Ms`, `ActivityKind`, `core/src/media.rs`, `Ui`, `Window`, `Settings`, `platform-win/src/media.rs`, `Point`, `Color`, `Shape`, `TrayIcon`, `Model`, `.layout_window`, `ViewKind`, `DeviceKind`, `Button`, `result`, `Lyrics`, `fswatch.rs`, `HKEY`, `Shelf`, `PowerReading`, `downloads.rs`, `.show_menu`, `platform-win/src/bluetooth.rs`, `Artwork`, `LyricStatus`, `Island`, `comp.rs`, `Config`, `log`, `UiProxy`, `PrivacyDot`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **Should `.handle` be split into smaller, more focused modules?**
  _Cohesion score 0.12807881773399016 - nodes in this community are weakly interconnected._
- **Why does `Event` connect `Event` to `window.rs`, `.handle`, `island/src/lib.rs`, `brightness.rs`, `core/src/media.rs`, `Window`, `settings.rs`, `HWND`, `platform-win/src/media.rs`, `Point`, `platform-win/src/privacy.rs`, `Lyrics`, `dragdrop.rs`, `fswatch.rs`, `HKEY`, `platform-win/src/bluetooth.rs`, `Artwork`, `comp.rs`, `log`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **Should `views.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.04591836734693878 - nodes in this community are weakly interconnected._