# Berkontribusi

Terima kasih sudah tertarik. Proyek ini masih pada tahap perencanaan; baca [`docs/PRD.md`](docs/PRD.md) sebelum mengusulkan perubahan besar.

## Aturan dasar

- **DCO, bukan CLA:** tanda tangani setiap commit dengan `git commit -s` (baris `Signed-off-by`).
- **Lisensi:** kontribusi kode dilisensikan Apache 2.0 (inbound = outbound, sesuai pasal 5 lisensi); kontribusi aset seni CC BY 4.0.
- **Kode `unsafe`:** hanya boleh berada di crate `platform-win`.
- **Tanpa komentar:** kode ditulis tanpa komentar (`//`, `///`, `//!`); nama fungsi dan tipe harus cukup menjelaskan.
- **Dependensi baru:** harus lolos `cargo deny` (hanya MIT, Apache-2.0, BSD, ISC, Unicode, OFL) dan dijelaskan alasannya di pull request.
- **Aset pihak lain:** jangan menyertakan ikon, font, logo, atau karakter milik pihak lain (termasuk Apple dan OpenAI).

Perilaku di ruang proyek mengikuti [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).

## Alur kontribusi

`main` dilindungi: tidak ada push langsung atau force push, dan setiap perubahan masuk lewat pull request dengan CI `windows-x64` yang hijau. Fork repositori ini, buat cabang, lalu buka pull request ke `main`. Pengaturan proteksinya dijelaskan di [`docs/KEAMANAN-REPO.md`](docs/KEAMANAN-REPO.md).
