# Graph Report - devnotch  (2026-11-04)

## Corpus Check
- 59 files · ~56,492 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 3, .toml 1)

## Summary
- 2102 nodes · 4187 edges · 99 communities (79 shown, 20 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 153 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a6795390`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- window.rs
- IslandApp
- views.rs
- Text
- gfx.rs
- Island
- config.rs
- island/src/lib.rs
- theme.rs
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
- fx.rs
- Json
- ui.rs
- HoverZone
- platform-win/src/privacy.rs
- comp.rs
- ViewKind
- TextStyle
- Palette
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
- Privacy
- Theme
- .lookup
- create_d3d
- 9. Rilis, pengujian, peta jalan, dan risiko
- v0.2.0.md
- fmt.rs
- 2. Spesifikasi visual dan catatan legal
- 3. Fitur Island (pil interaktif di tengah atas layar)
- v0.3.0.md
- .open
- PRD — Notchi: Island dan Dock untuk Windows 11 (Rust)
- 5. Menu bar, panel kontrol, launcher, dan pengaturan
- 6. Arsitektur teknis (Rust)
- 7. Keamanan dan privasi
- v0.1.0.md
- v0.2.1.md
- Notchi
- cli
- open_run_key
- init_process
- 4. Fitur Dock (pengganti taskbar bawah, Fase 3)
- smoke.ps1
- Artwork
- Hotkey
- InstanceLock
- notchi
- PrivacyDot
- PCWSTR
- icon.rs
- HoverZone
- Style
- DynText
- v0.4.0.md
- Panduan untuk agen AI
- Warning
- inside
- ReduceMotion
- has_family
- PANEL_EX
- PANEL_STYLE

## God Nodes (most connected - your core abstractions)
1. `IslandApp` - 119 edges
2. `Ms` - 103 edges
3. `Text` - 85 edges
4. `Ui` - 62 edges
5. `Rect` - 54 edges
6. `Event` - 50 edges
7. `Size` - 44 edges
8. `Settings` - 40 edges
9. `Island` - 35 edges
10. `Point` - 34 edges

## Surprising Connections (you probably didn't know these)
- `watch_dir()` --references--> `UiProxy`  [EXTRACTED]
  crates/platform-win/src/fswatch.rs → crates/platform-win/src/window.rs
- `EYE` --references--> `Color`  [EXTRACTED]
  crates/core/src/icon.rs → crates/core/src/color.rs
- `CAMERA_COLOR` --references--> `Color`  [EXTRACTED]
  crates/core/src/privacy.rs → crates/core/src/color.rs
- `MIC_COLOR` --references--> `Color`  [EXTRACTED]
  crates/core/src/privacy.rs → crates/core/src/color.rs
- `BLACK` --references--> `Color`  [EXTRACTED]
  crates/core/src/theme.rs → crates/core/src/color.rs

## Import Cycles
- None detected.

## Communities (99 total, 20 thin omitted)

### Community 0 - "window.rs"
Cohesion: 0.01
Nodes (13): CLASS_NAME, MAX_Z_WALK, PANEL_CLASS, PANEL_MIN_DP, quit(), WM_APP_APPBAR, WM_APP_ARRANGE, WM_APP_CAPTURE (+5 more)

### Community 1 - "IslandApp"
Cohesion: 0.06
Nodes (23): Point, Content, flag(), IslandApp, load_config(), log(), lyric_key(), LyricSlot (+15 more)

### Community 2 - "views.rs"
Cohesion: 0.05
Nodes (64): t(), ambang_sekali_per_siklus(), ambang_tidak_valid_dibuang(), bat(), BatteryMonitor, colok_dan_cabut(), mulai_dalam_kondisi_rendah(), pc_tanpa_baterai_diam() (+56 more)

### Community 3 - "Text"
Cohesion: 0.02
Nodes (84): Text, AboutBody, Advanced, AdvancedDesc, AlreadyRunning, AvoidWindows, BatteryCritical, BatteryLow (+76 more)

### Community 5 - "Island"
Cohesion: 0.05
Nodes (43): Priority, P0, P1, P2, P3, P4, ActiveAlert, alert_menyela_lalu_kembali_ke_state_semula() (+35 more)

### Community 6 - "config.rs"
Cohesion: 0.12
Nodes (30): bagian_bukan_tabel(), berkas_kosong_memakai_default(), berkas_terlalu_besar_ditolak(), daftar_dibatasi(), DEFAULT_HOTKEY, DEFAULT_TOML, finish(), is_key_line() (+22 more)

### Community 7 - "island/src/lib.rs"
Cohesion: 0.03
Nodes (49): COLLAPSE_DELAY_MS, CONTENT_DELAY_MS, CONTENT_FADE_MS, COVER_SETTLE_MS, DOT, DOT_INSET, DRAG_DP, FX_PAD (+41 more)

### Community 8 - "theme.rs"
Cohesion: 0.15
Nodes (12): aksen_tetap_terbaca_di_tema_terang(), all(), BLACK, INK, kaca_tembus_pandang(), klasik_datar_tanpa_bayangan(), NONE, on_color() (+4 more)

### Community 9 - "Ms"
Cohesion: 0.09
Nodes (25): Ms, ContentHold, batas_durasi(), Countdown, CountdownState, Paused, Running, hitung_mundur_dasar() (+17 more)

### Community 10 - "ActivityKind"
Cohesion: 0.05
Nodes (41): Activities, ActivityKind, Download, Media, Privacy, Stopwatch, Timer, carousel_berputar_tanpa_menggeser_p0() (+33 more)

### Community 11 - "layout.rs"
Cohesion: 0.07
Nodes (29): anchor_px(), BUBBLE_GAP, BUDDY_MASCOT, COMPACT_MAX_W, COMPACT_MIN_W, corner_radius(), EXPANDED_MAX_H, EXPANDED_MIN_H (+21 more)

### Community 12 - "core/src/media.rs"
Cohesion: 0.11
Nodes (22): art_fingerprint(), laporan_baru_dan_lagu_baru_memakai_timeline_segar(), laporan_posisi_rutin_dibuang(), nama_sumber(), PlaybackStatus, Changing, Closed, Opened (+14 more)

### Community 13 - "Event"
Cohesion: 0.06
Nodes (38): Event, Close, DisplayChanged, DpiChanged, FileChanged, ForegroundChanged, ForeignMouseDown, FullscreenHint (+30 more)

### Community 14 - "Ui"
Cohesion: 0.21
Nodes (7): Rect, Mascot, mascot(), Ui, waveform(), v2(), v3()

### Community 15 - "Window"
Cohesion: 0.08
Nodes (4): Handler, OwnedWindow, Window, WindowState

### Community 16 - "net.rs"
Cohesion: 0.06
Nodes (10): fetch_lyrics(), Handle, https_get(), json_from(), LYRICS_FALLBACK_HOST, LYRICS_HOST, lyrics_ovh(), LyricsFetch (+2 more)

### Community 17 - "Settings"
Cohesion: 0.15
Nodes (7): Config, sparkle_colors(), Action, behavior_desc(), Look, Settings, value()

### Community 19 - "settings.rs"
Cohesion: 0.07
Nodes (21): CARD_R, COL, Control, GAP, INNER, Kind, Button, Choice (+13 more)

### Community 20 - "release.yml"
Cohesion: 0.07
Nodes (29): concurrency, cancel-in-progress, group, run, CARGO_TERM_COLOR, GH_TOKEN, TAG, tag (+21 more)

### Community 21 - "Setting"
Cohesion: 0.07
Nodes (21): Setting, Advanced, AvoidWindows, Behavior, Boop, Fullscreen, HoverZone, Language (+13 more)

### Community 22 - "UiProxy"
Cohesion: 0.11
Nodes (16): main(), arrange_event_proc(), create_overlay(), error_box(), fill_menu(), frame_bounds(), is_app_window(), is_desktop() (+8 more)

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
Cohesion: 0.08
Nodes (10): MAX_ART_BYTES, COMMAND_SETTLE_MS, HOLD_MS, next_command(), play_pause(), POSITION_TOLERANCE_MS, read_thumbnail(), status_of() (+2 more)

### Community 27 - "lyrics.rs"
Cohesion: 0.12
Nodes (22): cadangan_lyrics_ovh(), clean_artist(), clean_title(), clip(), DURATION_SLACK_MS, indeks_dan_pergantian_tersinkron(), json_lengkap(), json_tidak_valid_ditolak() (+14 more)

### Community 29 - "State"
Cohesion: 0.20
Nodes (8): SourceFilter, aumid(), Props, read_props(), run(), State, Subscribed, UiProxy

### Community 30 - "MediaSnapshot"
Cohesion: 0.22
Nodes (5): Controls, media_dijeda_bertahan_sebentar(), MediaPresence, MediaSnapshot, ViewKey

### Community 31 - "Color"
Cohesion: 0.17
Nodes (12): AA, accent_alert(), accent_charging(), Color, ISLAND_BG, kontras_hitam_putih_21(), semua_token_lulus_wcag_aa_di_atas_hitam(), STATUS_CRITICAL (+4 more)

### Community 32 - "Shape"
Cohesion: 0.28
Nodes (14): bibir_dan_zona_dekat(), compact_size(), hit_box(), minimal_terpusat_dengan_gelembung(), near_zone(), pil_merapat_ke_tepi_tanpa_terpotong(), pill_center(), region_diam_pas_dengan_pil() (+6 more)

### Community 34 - "fx.rs"
Cohesion: 0.13
Nodes (10): Burst, burst(), Confetti, Pop, Sparkle, EASE_OUT, GRAVITY, Rng (+2 more)

### Community 35 - "Json"
Cohesion: 0.20
Nodes (8): Json, Arr, Bool, Null, Num, Obj, Str, Parser

### Community 36 - "ui.rs"
Cohesion: 0.10
Nodes (8): BLUSH, EYE, OVERLAP, Pill, Tone, Disabled, Neutral, Tinted

### Community 37 - "HoverZone"
Cohesion: 0.22
Nodes (5): Placement, .ALL, Center, Left, Right

### Community 39 - "comp.rs"
Cohesion: 0.07
Nodes (7): Animator, Batch, boxed_f32(), CompositionHost, LOOP_BUDGET_MS, REDUCED_MS, ts_ms()

### Community 40 - "ViewKind"
Cohesion: 0.18
Nodes (13): PeekKind, Idle, Media, ViewKind, Alert, Buddy, Card, Compact (+5 more)

### Community 41 - "TextStyle"
Cohesion: 0.15
Nodes (6): TextStyle, Weight, Bold, Medium, Regular, Semibold

### Community 42 - "Palette"
Cohesion: 0.31
Nodes (4): Palette, Relief, shade(), tint()

### Community 43 - "Size"
Cohesion: 0.18
Nodes (11): Size, BATTERY_BADGE, BUBBLE, BUDDY, compact_slots(), HUD, IDLE, PEEK_IDLE (+3 more)

### Community 44 - "Button"
Cohesion: 0.09
Nodes (26): alert_card(), AlertCard, Button, Lyrics, Mascot, Next, PlayPause, Preset (+18 more)

### Community 47 - "Lyrics"
Cohesion: 0.32
Nodes (5): from_record(), Lyrics, parse_ovh(), pick(), lrclib()

### Community 48 - "README.md"
Cohesion: 0.19
Nodes (7): Aturan dasar, Berkontribusi, Lisensi aset, Cakupan, Kebijakan keamanan, Melaporkan kerentanan, Kebijakan nama dan logo

### Community 49 - "Lang"
Cohesion: 0.21
Nodes (11): bahasa(), DAYS_EN, DAYS_ID, Lang, En, Id, long_date(), MONTHS_EN (+3 more)

### Community 50 - "Skin"
Cohesion: 0.20
Nodes (9): Skin, .ALL, Karang, Langit, Lavender, Lemon, Mint, Permen (+1 more)

### Community 52 - "HKEY"
Cohesion: 0.24
Nodes (9): Privacy, any_in_use(), capability(), Key, open(), qword(), read(), subkeys() (+1 more)

### Community 53 - "Card"
Cohesion: 0.25
Nodes (7): Card, Home, Lyrics, Media, Stopwatch, Timer, ukuran_sesuai_prd()

### Community 54 - "Query"
Cohesion: 0.24
Nodes (6): bare_title(), encode_component(), Query, query_apple_music_dan_feat(), query_dari_judul_video(), url_dikodekan()

### Community 55 - "Privacy"
Cohesion: 0.40
Nodes (4): CAMERA_COLOR, in_use(), MIC_COLOR, sedang_dipakai_bila_berhenti_lebih_lama_dari_mulai()

### Community 56 - "Theme"
Cohesion: 0.17
Nodes (8): Mode, Dark, Light, Theme, .ALL, Dark, Light, System

### Community 59 - "9. Rilis, pengujian, peta jalan, dan risiko"
Cohesion: 0.25
Nodes (8): 9.1 Peta jalan, 9.2 Pengemasan dan distribusi, 9.3 Strategi pengujian, 9.4 Risiko, 9.5 Definisi selesai untuk versi 1.0, 9.6 Pertanyaan terbuka, 9.7 Lisensi dan tata kelola open source, 9. Rilis, pengujian, peta jalan, dan risiko

### Community 60 - "v0.2.0.md"
Cohesion: 0.25
Nodes (7): Baru, Catatan, Island tidak lagi menghalangi tab browser, Pemutaran media, Perbaikan, Persyaratan, Unduhan

### Community 61 - "fmt.rs"
Cohesion: 0.60
Nodes (5): clock(), countdown(), elapsed(), format_berlalu(), format_hitung_mundur()

### Community 62 - "2. Spesifikasi visual dan catatan legal"
Cohesion: 0.29
Nodes (7): 2.1 Material kaca, 2.2 Bentuk, tipografi, ikon, 2.3 Gerak (motion), 2.4 Aksesibilitas, 2.5 Catatan legal dan kekayaan intelektual, 2.6 Maskot dan palet warna, 2. Spesifikasi visual dan catatan legal

### Community 63 - "3. Fitur Island (pil interaktif di tengah atas layar)"
Cohesion: 0.29
Nodes (7): 3.1 State dan ukuran, 3.2 Aktivitas live, 3.3 Interaksi, 3.4 Prioritas dan antrean, 3.5 Kriteria penerimaan Island, 3.6 Sumber media: YouTube Music, Apple Music, dan lainnya, 3. Fitur Island (pil interaktif di tengah atas layar)

### Community 64 - "v0.3.0.md"
Cohesion: 0.29
Nodes (6): Baru, Catatan, Perbaikan, Persyaratan, Perubahan perilaku, Unduhan

### Community 65 - ".open"
Cohesion: 0.23
Nodes (7): rect_contains_eksklusif_di_tepi_kanan(), scale_for_dpi(), skala_dpi(), create_hicon(), create_panel(), IconImage, mouse_point()

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
Cohesion: 0.24
Nodes (5): IslandConfig, Hotkey, parse_dasar(), vk_from_name(), vk_name()

### Community 82 - "notchi"
Cohesion: 1.00
Nodes (4): notchi, notchi-core, notchi-island, notchi-platform-win

### Community 85 - "icon.rs"
Cohesion: 0.42
Nodes (8): ellipse_inside(), EYE, mascot_rgba(), mata_gelap(), px(), rounded_rect_sdf(), sudut_transparan_tengah_opak(), ukuran_buffer_benar()

### Community 86 - "HoverZone"
Cohesion: 0.25
Nodes (4): HoverZone, .ALL, Normal, Small

### Community 87 - "Style"
Cohesion: 0.25
Nodes (6): Style, .ALL, Classic, Clay, Glass, Neumorphism

### Community 88 - "DynText"
Cohesion: 0.39
Nodes (6): Align, Center, Left, Right, DynText, place()

### Community 89 - "v0.4.0.md"
Cohesion: 0.33
Nodes (5): Baru, Catatan, Perbaikan, Persyaratan, Unduhan

### Community 90 - "Panduan untuk agen AI"
Cohesion: 0.40
Nodes (4): Cari file lewat Graphify dulu, Panduan untuk agen AI, Peta singkat, Wajib setelah mengubah kode

### Community 91 - "Warning"
Cohesion: 0.50
Nodes (4): check_unknown(), legacy_behavior(), Reader, Warning

### Community 92 - "inside"
Cohesion: 0.40
Nodes (5): inside(), pil_peek_muat_di_jendela(), seek_fraction(), seek_zone(), zona_seek_lebih_mudah_diraih_dari_bilah()

### Community 93 - "ReduceMotion"
Cohesion: 0.50
Nodes (4): ReduceMotion, Off, On, System

## Knowledge Gaps
- **261 isolated node(s):** `Loading`, `Missing`, `Download`, `Media`, `Privacy` (+256 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1056 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **20 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Text` connect `Text` to `Lang`, `views.rs`?**
  _High betweenness centrality (0.116) - this node is a cross-community bridge._
- **What connects `Loading`, `Missing`, `Download` to the rest of the system?**
  _261 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `window.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.010101010101010102 - nodes in this community are weakly interconnected._
- **Why does `Color` connect `Color` to `window.rs`, `IslandApp`, `views.rs`, `gfx.rs`, `island/src/lib.rs`, `theme.rs`, `Ui`, `Settings`, `settings.rs`, `Gfx`, `fx.rs`, `ui.rs`, `TextStyle`, `Palette`, `HKEY`, `Privacy`, `.open`, `PrivacyDot`, `icon.rs`?**
  _High betweenness centrality (0.094) - this node is a cross-community bridge._
- **Should `IslandApp` be split into smaller, more focused modules?**
  _Cohesion score 0.05536942338211955 - nodes in this community are weakly interconnected._
- **Why does `IslandApp` connect `IslandApp` to `views.rs`, `Island`, `island/src/lib.rs`, `Ms`, `ActivityKind`, `Ui`, `Window`, `Settings`, `MediaCommand`, `State`, `MediaSnapshot`, `Color`, `Shape`, `TrayIcon`, `ui.rs`, `comp.rs`, `ViewKind`, `Button`, `result`, `Lyrics`, `HKEY`, `Artwork`, `PrivacyDot`, `Warning`?**
  _High betweenness centrality (0.078) - this node is a cross-community bridge._
- **Should `views.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05239240844693932 - nodes in this community are weakly interconnected._