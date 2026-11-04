# Graph Report - devnotch  (2026-11-04)

## Corpus Check
- 64 files · ~61,617 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 3, .toml 1)

## Summary
- 2286 nodes · 4543 edges · 115 communities (91 shown, 24 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 166 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d6c5569d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- window.rs
- IslandApp
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
- UiProxy
- steps
- Gfx
- MediaCommand
- platform-win/src/media.rs
- lyrics.rs
- State
- MediaSnapshot
- Color
- Shape
- TrayIcon
- Model
- Json
- ui.rs
- HoverZone
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
- Skin
- fswatch.rs
- HKEY
- Card
- Query
- PowerReading
- downloads.rs
- .lookup
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
- Hotkey
- InstanceLock
- notchi
- PCWSTR
- Island
- timer.rs
- comp.rs
- DynText
- v0.4.0.md
- Panduan untuk agen AI
- AlertKind
- wndproc
- ReduceMotion
- Behavior
- semua_elemen_kartu_di_dalam_kartu
- PANEL_EX
- PANEL_STYLE
- Batch
- Presentation
- Expression
- Slot
- CompositionHost
- PlaybackStatus
- UiProxy
- Priority
- BluetoothWatcher
- LyricState
- sembunyi_otomatis_ke_tepi_atas
- MouseButton
- v0.5.0.md
- TrayEvent

## God Nodes (most connected - your core abstractions)
1. `IslandApp` - 128 edges
2. `Ms` - 112 edges
3. `Text` - 98 edges
4. `Ui` - 63 edges
5. `Event` - 60 edges
6. `Rect` - 55 edges
7. `Size` - 44 edges
8. `Settings` - 40 edges
9. `Island` - 35 edges
10. `Point` - 34 edges

## Surprising Connections (you probably didn't know these)
- `STALL_MS` --references--> `Ms`  [EXTRACTED]
  crates/core/src/downloads.rs → crates/core/src/lib.rs
- `RATE_SAMPLE_MS` --references--> `Ms`  [EXTRACTED]
  crates/core/src/downloads.rs → crates/core/src/lib.rs
- `build_alert()` --calls--> `countdown()`  [INFERRED]
  crates/island/src/views.rs → crates/core/src/fmt.rs
- `timer_text()` --calls--> `countdown()`  [INFERRED]
  crates/island/src/views.rs → crates/core/src/fmt.rs
- `stopwatch_text()` --calls--> `elapsed()`  [INFERRED]
  crates/island/src/views.rs → crates/core/src/fmt.rs

## Import Cycles
- None detected.

## Communities (115 total, 24 thin omitted)

### Community 0 - "window.rs"
Cohesion: 0.01
Nodes (13): CLASS_NAME, MAX_Z_WALK, PANEL_CLASS, PANEL_MIN_DP, quit(), WM_APP_APPBAR, WM_APP_ARRANGE, WM_APP_CAPTURE (+5 more)

### Community 1 - "IslandApp"
Cohesion: 0.05
Nodes (24): Point, Content, flag(), IslandApp, load_config(), log(), lyric_key(), LyricSlot (+16 more)

### Community 2 - "views.rs"
Cohesion: 0.06
Nodes (34): lyric_timing(), clock_text(), download_detail(), download_short(), DownloadInfo, G_BLUETOOTH, G_BRIGHTNESS, G_CANCEL (+26 more)

### Community 3 - "Text"
Cohesion: 0.02
Nodes (97): Text, AboutBody, Advanced, AdvancedDesc, AlreadyRunning, AvoidWindows, BatteryCritical, BatteryLow (+89 more)

### Community 5 - "island.rs"
Cohesion: 0.22
Nodes (14): alert_menyela_lalu_kembali_ke_state_semula(), alert_prioritas_rendah_tidak_menimpa(), charging(), hide_muncul_hanya_saat_aktivitas(), hud_volume_tidak_menutup_kartu(), jumlah_aktivitas_menentukan_bentuk(), klik_alert_membuka_kartu(), klik_expand_dan_collapse() (+6 more)

### Community 6 - "config.rs"
Cohesion: 0.12
Nodes (32): bagian_bukan_tabel(), berkas_kosong_memakai_default(), berkas_terlalu_besar_ditolak(), check_unknown(), daftar_dibatasi(), DEFAULT_HOTKEY, DEFAULT_TOML, finish() (+24 more)

### Community 7 - "island/src/lib.rs"
Cohesion: 0.03
Nodes (54): BRIGHTNESS_STEP, COLLAPSE_DELAY_MS, CONTENT_DELAY_MS, CONTENT_FADE_MS, COVER_SETTLE_MS, DOT, DOT_INSET, DOWNLOAD_DEBOUNCE_MS (+46 more)

### Community 8 - "brightness.rs"
Cohesion: 0.07
Nodes (16): apply(), Backend, Ddc, None, Wmi, DDC_CACHE, ddc_read(), ddc_step() (+8 more)

### Community 9 - "Ms"
Cohesion: 0.15
Nodes (12): Ms, ContentHold, Countdown, CountdownState, Paused, Running, HOUR_MS, LYRIC_LEAD_MS (+4 more)

### Community 10 - "ActivityKind"
Cohesion: 0.10
Nodes (23): Activities, ActivityKind, Download, Media, Privacy, Stopwatch, Timer, carousel_berputar_tanpa_menggeser_p0() (+15 more)

### Community 11 - "layout.rs"
Cohesion: 0.07
Nodes (36): anchor_px(), BUBBLE_GAP, BUDDY_MASCOT, COMPACT_MAX_W, COMPACT_MIN_W, corner_radius(), EXPANDED_MAX_H, EXPANDED_MIN_H (+28 more)

### Community 12 - "core/src/media.rs"
Cohesion: 0.15
Nodes (16): art_fingerprint(), laporan_baru_dan_lagu_baru_memakai_timeline_segar(), laporan_posisi_rutin_dibuang(), media_dijeda_bertahan_sebentar(), nama_sumber(), posisi_diekstrapolasi_saat_diputar(), posisi_tidak_melompat_saat_timeline_basi(), RawTimeline (+8 more)

### Community 13 - "Event"
Cohesion: 0.08
Nodes (26): Event, Bluetooth, BluetoothReady, BluetoothRemoved, Brightness, Close, DisplayChanged, DownloadsChanged (+18 more)

### Community 14 - "Ui"
Cohesion: 0.15
Nodes (9): Rect, rect_contains_eksklusif_di_tepi_kanan(), Mascot, mascot(), Pill, Ui, waveform(), v2() (+1 more)

### Community 15 - "Window"
Cohesion: 0.08
Nodes (4): Handler, OwnedWindow, Window, WindowState

### Community 16 - "net.rs"
Cohesion: 0.07
Nodes (6): Handle, https_get(), LYRICS_FALLBACK_HOST, LYRICS_HOST, Response, user_agent()

### Community 17 - "Settings"
Cohesion: 0.12
Nodes (10): BluetoothConfig, Config, DownloadsConfig, MediaConfig, PowerConfig, TimerConfig, behavior_desc(), Look (+2 more)

### Community 18 - "shell.rs"
Cohesion: 0.06
Nodes (4): local_time(), LocalTime, open_folder(), reveal()

### Community 19 - "settings.rs"
Cohesion: 0.06
Nodes (26): Burst, Confetti, Pop, Sparkle, Spark, CARD_R, COL, Control (+18 more)

### Community 20 - "release.yml"
Cohesion: 0.07
Nodes (29): concurrency, cancel-in-progress, group, run, CARGO_TERM_COLOR, GH_TOKEN, TAG, tag (+21 more)

### Community 21 - "Setting"
Cohesion: 0.08
Nodes (24): Setting, Advanced, AvoidWindows, Behavior, Bluetooth, Boop, Downloads, Fullscreen (+16 more)

### Community 22 - "UiProxy"
Cohesion: 0.11
Nodes (17): main(), arrange_event_proc(), create_overlay(), error_box(), fill_menu(), frame_bounds(), is_app_window(), is_desktop() (+9 more)

### Community 23 - "steps"
Cohesion: 0.07
Nodes (28): concurrency, cancel-in-progress, group, run, CARGO_TERM_COLOR, jobs, windows-x64, name (+20 more)

### Community 24 - "Gfx"
Cohesion: 0.13
Nodes (3): d2d_color(), Gfx, Surface

### Community 25 - "MediaCommand"
Cohesion: 0.12
Nodes (13): MediaCommand, CurrentChanged, CycleSession, Next, Prev, Quick, Refresh, Seek (+5 more)

### Community 26 - "platform-win/src/media.rs"
Cohesion: 0.09
Nodes (10): MAX_ART_BYTES, COMMAND_SETTLE_MS, HOLD_MS, play_pause(), POSITION_TOLERANCE_MS, Props, read_props(), read_thumbnail() (+2 more)

### Community 27 - "lyrics.rs"
Cohesion: 0.11
Nodes (23): cadangan_lyrics_ovh(), clean_artist(), clean_title(), clip(), DURATION_SLACK_MS, indeks_dan_pergantian_tersinkron(), json_lengkap(), json_tidak_valid_ditolak() (+15 more)

### Community 29 - "State"
Cohesion: 0.21
Nodes (6): SourceFilter, aumid(), run(), State, Subscribed, windows_filetime_now()

### Community 31 - "Color"
Cohesion: 0.05
Nodes (43): AA, accent_alert(), accent_charging(), Color, ISLAND_BG, kontras_hitam_putih_21(), semua_token_lulus_wcag_aa_di_atas_hitam(), STATUS_CRITICAL (+35 more)

### Community 32 - "Shape"
Cohesion: 0.28
Nodes (14): bibir_dan_zona_dekat(), compact_size(), hit_box(), minimal_terpusat_dengan_gelembung(), near_zone(), pil_merapat_ke_tepi_tanpa_terpotong(), pill_center(), region_diam_pas_dengan_pil() (+6 more)

### Community 33 - "TrayIcon"
Cohesion: 0.14
Nodes (6): colorref(), create_hicon(), create_panel(), IconImage, set_caption(), TrayIcon

### Community 34 - "Model"
Cohesion: 0.28
Nodes (20): t(), battery_badge(), build_alert(), build_bubble(), build_buddy(), build_compact(), build_download_card(), build_home_card() (+12 more)

### Community 35 - "Json"
Cohesion: 0.20
Nodes (8): Json, Arr, Bool, Null, Num, Obj, Str, Parser

### Community 36 - "ui.rs"
Cohesion: 0.07
Nodes (14): TEXT_PRIMARY, burst(), EASE_OUT, GRAVITY, Rng, SOFT, sparkle_colors(), BLUSH (+6 more)

### Community 37 - "HoverZone"
Cohesion: 0.11
Nodes (10): HoverZone, .ALL, Normal, Small, is_side(), Placement, .ALL, Center (+2 more)

### Community 40 - "ViewKind"
Cohesion: 0.14
Nodes (15): Controls, PeekKind, Idle, Media, ViewKey, ViewKind, Alert, Buddy (+7 more)

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
Cohesion: 0.12
Nodes (17): Button, Lyrics, Mascot, Next, OpenDownloads, PlayPause, Preset, Prev (+9 more)

### Community 47 - "Lyrics"
Cohesion: 0.22
Nodes (10): from_record(), Line, Lyrics, pick(), pilih_dari_lrclib(), fetch_lyrics(), json_from(), lrclib() (+2 more)

### Community 48 - "README.md"
Cohesion: 0.19
Nodes (7): Aturan dasar, Berkontribusi, Lisensi aset, Cakupan, Kebijakan keamanan, Melaporkan kerentanan, Kebijakan nama dan logo

### Community 49 - "Lang"
Cohesion: 0.20
Nodes (12): bahasa(), DAYS_EN, DAYS_ID, decimal(), Lang, En, Id, long_date() (+4 more)

### Community 50 - "Skin"
Cohesion: 0.15
Nodes (17): Skin, .ALL, Karang, Langit, Lavender, Lemon, Mint, Permen (+9 more)

### Community 52 - "HKEY"
Cohesion: 0.24
Nodes (9): Privacy, any_in_use(), capability(), Key, open(), qword(), read(), subkeys() (+1 more)

### Community 53 - "Card"
Cohesion: 0.29
Nodes (7): Card, Download, Home, Lyrics, Media, Stopwatch, Timer

### Community 54 - "Query"
Cohesion: 0.36
Nodes (4): bare_title(), encode_component(), Query, query_apple_music_dan_feat()

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
Cohesion: 0.13
Nodes (5): BATTERY, CLASSIC, CONNECTED, LE, props_of()

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

### Community 80 - "Hotkey"
Cohesion: 0.17
Nodes (7): IslandConfig, scale_for_dpi(), skala_dpi(), Hotkey, parse_dasar(), vk_from_name(), vk_name()

### Community 82 - "notchi"
Cohesion: 1.00
Nodes (4): notchi, notchi-core, notchi-island, notchi-platform-win

### Community 86 - "timer.rs"
Cohesion: 0.20
Nodes (9): batas_durasi(), hitung_mundur_dasar(), jeda_dan_lanjut(), kurangi_lewat_scroll_tidak_menamatkan_timer(), MAX_COUNTDOWN, Stopwatch, stopwatch(), tambah_satu_menit() (+1 more)

### Community 87 - "comp.rs"
Cohesion: 0.13
Nodes (3): boxed_f32(), LOOP_BUDGET_MS, REDUCED_MS

### Community 88 - "DynText"
Cohesion: 0.39
Nodes (6): Align, Center, Left, Right, DynText, place()

### Community 89 - "v0.4.0.md"
Cohesion: 0.33
Nodes (5): Baru, Catatan, Perbaikan, Persyaratan, Unduhan

### Community 90 - "Panduan untuk agen AI"
Cohesion: 0.40
Nodes (4): Cari file lewat Graphify dulu, Panduan untuk agen AI, Peta singkat, Wajib setelah mengubah kode

### Community 91 - "AlertKind"
Cohesion: 0.15
Nodes (11): ActiveAlert, AlertKind, Bluetooth, Brightness, Charging, DownloadDone, LowBattery, TimerDone (+3 more)

### Community 92 - "wndproc"
Cohesion: 0.17
Nodes (11): DpiChanged, FullscreenHint, Key, MouseDown, MouseMove, MouseUp, ShiftWheel, Timer (+3 more)

### Community 93 - "ReduceMotion"
Cohesion: 0.50
Nodes (4): ReduceMotion, Off, On, System

### Community 94 - "Behavior"
Cohesion: 0.25
Nodes (5): Behavior, .ALL, Autohide, Hide, Sticky

### Community 95 - "semua_elemen_kartu_di_dalam_kartu"
Cohesion: 0.25
Nodes (8): alert_card(), AlertCard, download_card(), DownloadCard, semua_elemen_kartu_di_dalam_kartu(), timer_card(), TimerCard, ukuran_sesuai_prd()

### Community 101 - "Presentation"
Cohesion: 0.29
Nodes (7): Presentation, Alert, Compact, Expanded, Hidden, Idle, Minimal

### Community 102 - "Expression"
Cohesion: 0.25
Nodes (8): Expression, Charging, Idle, Listening, Neutral, Sleepy, Surprised, Watching

### Community 103 - "Slot"
Cohesion: 0.29
Nodes (8): slot_of(), compact_size(), compact_width(), Slot, Download, Media, Stopwatch, Timer

### Community 105 - "PlaybackStatus"
Cohesion: 0.29
Nodes (7): PlaybackStatus, Changing, Closed, Opened, Paused, Playing, Stopped

### Community 106 - "UiProxy"
Cohesion: 0.38
Nodes (4): Brightness, worker(), next_command(), UiProxy

### Community 107 - "Priority"
Cohesion: 0.33
Nodes (6): Priority, P0, P1, P2, P3, P4

### Community 109 - "LyricState"
Cohesion: 0.40
Nodes (5): LyricState, Loading, Missing, Offline, Ready

### Community 111 - "MouseButton"
Cohesion: 0.50
Nodes (4): MouseButton, Left, Middle, Right

### Community 112 - "v0.5.0.md"
Cohesion: 0.50
Nodes (3): Baru, Catatan, Unduhan

### Community 113 - "TrayEvent"
Cohesion: 0.67
Nodes (3): TrayEvent, Click, Menu

## Knowledge Gaps
- **296 isolated node(s):** `Audio`, `Mouse`, `Keyboard`, `Phone`, `Controller` (+291 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1143 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **24 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Text` connect `Text` to `Lang`, `Model`?**
  _High betweenness centrality (0.092) - this node is a cross-community bridge._
- **What connects `Audio`, `Mouse`, `Keyboard` to the rest of the system?**
  _296 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `window.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.01020408163265306 - nodes in this community are weakly interconnected._
- **Why does `Event` connect `Event` to `window.rs`, `IslandApp`, `island/src/lib.rs`, `brightness.rs`, `Window`, `settings.rs`, `UiProxy`, `platform-win/src/media.rs`, `MediaSnapshot`, `platform-win/src/privacy.rs`, `Lyrics`, `fswatch.rs`, `HKEY`, `platform-win/src/bluetooth.rs`, `Artwork`, `comp.rs`, `wndproc`, `UiProxy`, `MouseButton`, `TrayEvent`?**
  _High betweenness centrality (0.089) - this node is a cross-community bridge._
- **Should `IslandApp` be split into smaller, more focused modules?**
  _Cohesion score 0.05246523388116309 - nodes in this community are weakly interconnected._
- **Why does `Color` connect `Color` to `window.rs`, `IslandApp`, `views.rs`, `Model`, `ui.rs`, `gfx.rs`, `TrayIcon`, `island/src/lib.rs`, `TextStyle`, `Ui`, `Settings`, `Skin`, `settings.rs`, `HKEY`, `Gfx`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Should `views.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05537098560354374 - nodes in this community are weakly interconnected._