# wa-lite

WhatsApp Web dalam jendela native yang ringan (Rust + wry), terkunci ke `web.whatsapp.com`.
Linux memakai WebKitGTK, Windows memakai WebView2. Versi library dikunci: wry 0.44.1 + tao 0.30.0.

## Fitur
- Ringan: tanpa Chromium sendiri, CPU idle mendekati 0%.
- Sesi login tersimpan (tidak perlu scan QR berulang).
- **Mode offline:** jika dibuka tanpa internet, aplikasi tetap mencoba memuat WhatsApp Web
  dari cache/data lokalnya sendiri (chat lama mungkin tampil read-only, tergantung WhatsApp).
  Jika setelah ~8 detik tidak tampil, muncul halaman offline. Saat koneksi kembali, otomatis
  pindah/segarkan ke WhatsApp Web online. Jika putus saat dipakai, tampilan terakhir
  dipertahankan dengan spanduk kecil. Pesan tetap tidak bisa dikirim/diterima selama offline.
- Ikon aplikasi kustom (jendela, `.exe`, launcher Linux).
- **Notifikasi desktop** (lihat catatan per OS di bawah).
- Tautan di luar web.whatsapp.com dibuka di browser bawaan sistem.

## Arch Linux (CLI)

1. Dependensi:

       sudo pacman -S --needed rust base-devel pkgconf gtk3 webkit2gtk-4.1 xdg-utils libnotify

   Notifikasi tampil lewat `notify-send` (paket `libnotify`) dan butuh notification daemon.
   GNOME dan KDE sudah punya; di window manager lain pasang `dunst` atau `mako`.

2. Build:

       unzip -o wa-lite.zip && cd wa-lite
       cargo build --release

3. Pasang:

       install -Dm755 target/release/wa-lite ~/.local/bin/wa-lite
       install -Dm644 wa-lite.desktop ~/.local/share/applications/wa-lite.desktop
       install -Dm644 assets/icon.png ~/.local/share/icons/hicolor/256x256/apps/wa-lite.png

   Pastikan `~/.local/bin` ada di `PATH`. Jalankan `wa-lite` atau cari "WhatsApp (wa-lite)" di launcher.

4. Notifikasi: izin otomatis diberikan. Setelah login, di WhatsApp buka
   Pengaturan -> Notifikasi dan pastikan aktif.

Data sesi: `~/.local/share/wa-lite`. Reset login: hapus folder itu.

## Windows 11 (VS Code)

1. Pasang "Build Tools for Visual Studio" dengan workload **Desktop development with C++**
   (sudah termasuk Windows SDK yang dibutuhkan untuk menyematkan ikon).
2. Pasang Rust dari rustup.rs (toolchain MSVC). Pasang ekstensi **rust-analyzer** di VS Code.
3. Buka folder `wa-lite` di VS Code, lalu di terminal:

       cargo build --release

4. Hasil: `target\release\wa-lite.exe` (sudah ber-ikon). WebView2 Runtime sudah bawaan Windows 11.

Notifikasi di Windows memakai fitur notifikasi bawaan WebView2: klik **Izinkan** pada
permintaan izin notifikasi yang muncul sekali, dan pastikan notifikasi Windows / Fokus tidak
sedang memblokir. Data sesi: `%LOCALAPPDATA%\wa-lite`.

## Pemecahan masalah
- **Layar putih/crash di Wayland/NVIDIA:** `WEBKIT_DISABLE_DMABUF_RENDERER=1 wa-lite`
  (permanen: ubah `Exec=` di `.desktop` menjadi `Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 wa-lite`).
- **`webkit2gtk-4.1` tidak ditemukan:** pastikan paket `webkit2gtk-4.1` terpasang.
- **Windows: gagal menyematkan ikon (`rc.exe`):** pasang Windows SDK lewat Build Tools,
  atau hapus `build.rs` dan bagian `[target.'cfg(windows)'.build-dependencies]` di `Cargo.toml`
  (ikon jendela tetap tampil, hanya ikon file `.exe` yang default).
- **Uninstall (Linux):** `rm ~/.local/bin/wa-lite ~/.local/share/applications/wa-lite.desktop ~/.local/share/icons/hicolor/256x256/apps/wa-lite.png`
