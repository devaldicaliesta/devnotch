# Mengamankan repositori

Langkah ini dilakukan sekali oleh pemilik repositori di github.com, karena pengaturan proteksi tidak bisa diubah lewat berkas di repositori. Berkas aturan yang siap diimpor ada di [`.github/rulesets/`](../.github/rulesets).

## Siapa yang bisa push?

Di repositori publik, orang lain **tidak bisa push** sama sekali kecuali Anda menambahkannya sebagai kolaborator. Mereka hanya bisa fork lalu membuka pull request, dan PR itu baru masuk bila Anda menggabungkannya. Aturan di bawah melindungi `main` dan tag rilis bahkan dari kesalahan kolaborator, aplikasi, atau token yang bocor.

## 1. Impor aturan cabang `main`

1. Buka **Settings → Rules → Rulesets**.
2. Pilih **New ruleset → Import a ruleset**, lalu unggah `.github/rulesets/main.json`.
3. Periksa, lalu **Create**.

Isinya:

- `main` tidak bisa dihapus dan tidak bisa di-force push.
- Setiap perubahan ke `main` harus lewat pull request.
- CI `windows-x64` harus hijau sebelum digabung.
- Admin (Anda) hanya bisa melewati aturan lewat pull request, bukan dengan push langsung.

Jumlah persetujuan wajib sengaja 0, karena pemilik tidak bisa menyetujui PR miliknya sendiri. Bila nanti ada pengelola lain, naikkan ke 1.

## 2. Impor aturan tag rilis

Ulangi langkah di atas dengan `.github/rulesets/tags.json`. Tag `v*` yang sudah ada tidak bisa dihapus, digeser, atau ditimpa, sehingga unduhan rilis lama tetap cocok dengan kodenya. Membuat tag baru (oleh workflow **Release**) tetap diizinkan.

## 3. Pengaturan lain yang disarankan

- **Settings → Actions → General → Fork pull request workflows from outside collaborators:** pilih **Require approval for all external contributors**, agar CI dari fork orang asing tidak berjalan sebelum Anda melihat kodenya.
- **Settings → Actions → General → Workflow permissions:** pilih **Read repository contents and packages permissions**. Workflow yang butuh menulis (Release) sudah meminta `contents: write` sendiri.
- **Settings → General → Pull Requests:** aktifkan **Automatically delete head branches**.
- **Settings → Code security:** aktifkan **Private vulnerability reporting** (dipakai oleh [`SECURITY.md`](../SECURITY.md)), **Dependabot alerts**, dan **Secret scanning / Push protection**.
- **Akun Anda:** aktifkan autentikasi dua faktor (2FA) dan tinjau aplikasi serta token di **Settings → Applications** dan **Developer settings → Personal access tokens**.

## Cara kerja setelah aturan aktif

- Kontributor: fork → cabang → pull request ke `main`. [`CODEOWNERS`](../.github/CODEOWNERS) otomatis meminta tinjauan Anda.
- Pemilik: tetap lewat pull request; gabungkan setelah CI hijau, lalu jalankan workflow **Release**.
