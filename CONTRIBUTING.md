# Berkontribusi

Terima kasih sudah tertarik. Proyek ini masih pada tahap perencanaan; baca [`docs/PRD.md`](docs/PRD.md) sebelum mengusulkan perubahan besar.

## Aturan dasar

- **DCO, bukan CLA:** tanda tangani setiap commit dengan `git commit -s` (baris `Signed-off-by`).
- **Lisensi:** kontribusi kode dilisensikan MIT; kontribusi aset seni CC BY 4.0.
- **Kode `unsafe`:** hanya boleh berada di crate `platform-win`.
- **Tanpa komentar:** kode ditulis tanpa komentar (`//`, `///`, `//!`); nama fungsi dan tipe harus cukup menjelaskan.
- **Dependensi baru:** harus lolos `cargo deny` (hanya MIT, Apache-2.0, BSD, ISC, Unicode, OFL) dan dijelaskan alasannya di pull request.
- **Aset pihak lain:** jangan menyertakan ikon, font, logo, atau karakter milik pihak lain (termasuk Apple dan OpenAI).

Perilaku di ruang proyek mengikuti [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
