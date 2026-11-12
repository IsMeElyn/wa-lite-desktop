# wa-lite

WhatsApp Web dalam jendela native yang ringan (Rust + wry). Terkunci ke `web.whatsapp.com`.
Di Linux memakai WebKitGTK, di Windows memakai WebView2.

> Catatan: dikunci ke wry 0.44.1 + tao 0.30.0 (API-nya berbeda dari versi terbaru).

## Instalasi di Arch Linux (CLI)

### 1. Pasang dependensi

    sudo pacman -S --needed rust base-devel pkgconf gtk3 webkit2gtk-4.1 xdg-utils

(Boleh pakai `rustup` sebagai ganti paket `rust`: `sudo pacman -S rustup && rustup default stable`.)

### 2. Ekstrak dan build

    unzip wa-lite.zip
    cd wa-lite
    cargo build --release

Build pertama mengunduh crate dan cukup lama. Hasilnya: `target/release/wa-lite`.

### 3. Pasang ke sistem (per-user)

    install -Dm755 target/release/wa-lite ~/.local/bin/wa-lite
    install -Dm644 wa-lite.desktop ~/.local/share/applications/wa-lite.desktop

Pastikan `~/.local/bin` ada di `PATH`. Jalankan dengan `wa-lite` dari terminal,
atau cari "WhatsApp (wa-lite)" di launcher aplikasi.

### 4. Pertama kali dipakai

Scan QR dari HP (Perangkat tertaut). Sesi disimpan di `~/.local/share/wa-lite`,
jadi tidak perlu scan ulang.

## Pemecahan masalah

- **Layar putih/hitam atau crash di Wayland/NVIDIA:** coba
  `WEBKIT_DISABLE_DMABUF_RENDERER=1 wa-lite`. Jika berhasil, tambahkan ke baris `Exec=`
  di file `.desktop` menjadi `Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 wa-lite`.
- **Error `webkit2gtk-4.1` tidak ditemukan saat build:** pastikan paket `webkit2gtk-4.1`
  terpasang (bukan hanya `webkit2gtk`).
- **WhatsApp bilang browser tidak didukung:** aplikasi sudah memakai User-Agent Chrome;
  jika masih muncul, kabari agar versinya diperbarui.
- **Reset login:** `rm -rf ~/.local/share/wa-lite`.
- **Uninstall:** `rm ~/.local/bin/wa-lite ~/.local/share/applications/wa-lite.desktop`.

## Catatan

- Butuh internet; ini hanya pembungkus WhatsApp Web, bukan klien offline.
- Tautan di luar `web.whatsapp.com` dibuka lewat `xdg-open`.
- Tidak ada polling: CPU idle mendekati 0%.

## Build di Windows

Pasang Rust (MSVC) + WebView2 Runtime, lalu `cargo build --release`.
Hasil: `target\release\wa-lite.exe`. Sesi disimpan di `%LOCALAPPDATA%\wa-lite`.
