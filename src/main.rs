// Sembunyikan jendela konsol pada build release di Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    thread,
    time::Duration,
};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Icon, WindowBuilder},
};
use wry::{WebContext, WebViewBuilder};

const HOME: &str = "https://web.whatsapp.com/";
const OFFLINE_HTML: &str = include_str!("../assets/offline.html");
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon.rgba"); // 128x128 RGBA
#[cfg(target_os = "linux")]
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

// WhatsApp Web menolak browser yang tidak dikenal, jadi di Linux (WebKitGTK)
// kita memakai User-Agent Chrome.
#[cfg(target_os = "linux")]
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
Chrome/124.0.0.0 Safari/537.36";

// Linux: WebKitGTK tidak menampilkan notifikasi web dengan sendirinya.
// Objek Notification digantikan jembatan yang memanggil `notify-send`.
#[cfg(target_os = "linux")]
const NOTIFY_JS: &str = r#"
(function () {
  if (!window.ipc) return;
  function N(title, opts) {
    this.title = String(title);
    var body = (opts && opts.body) ? String(opts.body) : '';
    window.ipc.postMessage('notify\u0001' + this.title + '\u0001' + body);
  }
  N.prototype.close = function () {};
  N.prototype.addEventListener = function () {};
  N.prototype.removeEventListener = function () {};
  N.permission = 'granted';
  N.requestPermission = function (cb) {
    if (typeof cb === 'function') cb('granted');
    return Promise.resolve('granted');
  };
  try {
    Object.defineProperty(window, 'Notification', { value: N, configurable: true, writable: true });
  } catch (e) {}
})();
"#;

const OFFLINE_BANNER_JS: &str = r#"
(function () {
  if (document.getElementById('wa-lite-off') || !document.body) return;
  var d = document.createElement('div');
  d.id = 'wa-lite-off';
  d.textContent = 'Offline — menyambung ulang otomatis…';
  d.style.cssText = 'position:fixed;left:0;right:0;bottom:0;z-index:2147483647;' +
    'background:#d6336c;color:#fff;font:13px sans-serif;text-align:center;' +
    'padding:6px;pointer-events:none';
  document.body.appendChild(d);
})();
"#;

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    Online,
    Offline,
}

/// Hanya web.whatsapp.com yang boleh dimuat di jendela ini.
fn allowed(url: &str) -> bool {
    url.starts_with("https://web.whatsapp.com")
        || url.starts_with("blob:https://web.whatsapp.com")
        || url.starts_with("about:")
        || url.starts_with("data:")
}

/// Tautan lain dibuka di browser bawaan sistem.
fn open_external(url: &str) {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return;
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Folder data (login, cache) agar sesi tidak hilang setiap aplikasi ditutup.
fn data_dir() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    #[cfg(not(windows))]
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("wa-lite")
}

/// Cek koneksi: buka TCP singkat ke web.whatsapp.com:443 (tanpa dependensi tambahan).
fn is_online() -> bool {
    let addrs = match ("web.whatsapp.com", 443).to_socket_addrs() {
        Ok(a) => a,
        Err(_) => return false,
    };
    for a in addrs {
        if TcpStream::connect_timeout(&a, Duration::from_secs(4)).is_ok() {
            return true;
        }
    }
    false
}

/// Thread pemantau koneksi. Online: cek tiap 20 dtk (5 dtk jika baru gagal).
/// Offline: cek tiap 3 dtk. Perubahan status dikirim ke event loop.
fn spawn_monitor(proxy: EventLoopProxy<UserEvent>, start_online: bool) {
    thread::spawn(move || {
        let mut online = start_online;
        let mut fails: u32 = if online { 0 } else { 2 };
        loop {
            let wait = if !online {
                3
            } else if fails > 0 {
                5
            } else {
                20
            };
            thread::sleep(Duration::from_secs(wait));
            if is_online() {
                fails = 0;
                if !online {
                    online = true;
                    if proxy.send_event(UserEvent::Online).is_err() {
                        break;
                    }
                }
            } else {
                fails += 1;
                if online && fails >= 2 {
                    online = false;
                    if proxy.send_event(UserEvent::Offline).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

#[cfg(target_os = "linux")]
fn notify(title: &str, body: &str, icon: &std::path::Path) {
    let title: String = title.chars().take(120).collect();
    let body: String = body.chars().take(400).collect();
    let icon = icon.to_path_buf();
    thread::spawn(move || {
        let _ = std::process::Command::new("notify-send")
            .args(["-a", "WhatsApp", "-i"])
            .arg(icon)
            .arg("--")
            .arg(title)
            .arg(body)
            .status();
    });
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let icon = Icon::from_rgba(ICON_RGBA.to_vec(), 128, 128).expect("ikon tidak valid");
    let window = WindowBuilder::new()
        .with_title("WhatsApp")
        .with_window_icon(Some(icon))
        .with_inner_size(LogicalSize::new(1100.0, 760.0))
        .with_min_inner_size(LogicalSize::new(420.0, 520.0))
        .build(&event_loop)
        .expect("gagal membuat jendela");

    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(target_os = "linux")]
    let icon_path = {
        let p = dir.join("icon.png");
        let _ = std::fs::write(&p, ICON_PNG); // untuk ikon notifikasi
        p
    };
    let mut context = WebContext::new(Some(dir));

    // Jika offline saat dibuka: tampilkan halaman offline, lalu otomatis
    // pindah ke WhatsApp Web begitu koneksi kembali.
    let online_at_start = is_online();
    spawn_monitor(proxy, online_at_start);

    // wry 0.44: di Linux builder dibuat dari GTK vbox (Wayland + X11).
    #[cfg(target_os = "linux")]
    let builder = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().expect("gagal mengambil GTK vbox");
        WebViewBuilder::new_gtk(vbox)
    };
    #[cfg(not(target_os = "linux"))]
    let builder = WebViewBuilder::new(&window);

    let builder = builder
        .with_web_context(&mut context)
        .with_navigation_handler(|url: String| {
            if allowed(&url) {
                true
            } else {
                open_external(&url);
                false
            }
        })
        .with_new_window_req_handler(|url: String| {
            if !allowed(&url) {
                open_external(&url);
            }
            false // jangan pernah buka jendela baru
        })
        .with_devtools(false);

    let builder = if online_at_start {
        builder.with_url(HOME)
    } else {
        builder.with_html(OFFLINE_HTML)
    };

    #[cfg(target_os = "linux")]
    let builder = builder
        .with_user_agent(UA)
        .with_initialization_script(NOTIFY_JS)
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            let mut it = req.body().splitn(3, '\u{1}');
            if it.next() == Some("notify") {
                let title = it.next().unwrap_or("");
                let body = it.next().unwrap_or("");
                notify(title, body, &icon_path);
            }
        });

    // Argumen Chromium (WebView2) untuk menekan aktivitas latar belakang.
    #[cfg(windows)]
    let builder = {
        use wry::WebViewBuilderExtWindows;
        builder.with_additional_browser_args(
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Translate \
             --disable-background-networking --disable-component-update \
             --disable-sync --disable-breakpad",
        )
    };

    let webview = builder.build().expect("gagal membuat webview");
    let mut on_offline_page = !online_at_start;

    event_loop.run(move |event, _, control_flow| {
        let _keep_alive = (&context, &window);
        *control_flow = ControlFlow::Wait; // tidak polling = CPU idle ~0%
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::UserEvent(UserEvent::Online) => {
                if on_offline_page {
                    on_offline_page = false;
                    let _ = webview.load_url(HOME);
                } else {
                    // Koneksi kembali setelah putus: segarkan ke WhatsApp Web online.
                    let _ = webview.evaluate_script("location.reload()");
                }
            }
            Event::UserEvent(UserEvent::Offline) => {
                // Tampilan WhatsApp terakhir tetap dipertahankan + spanduk kecil.
                if !on_offline_page {
                    let _ = webview.evaluate_script(OFFLINE_BANNER_JS);
                }
            }
            _ => {}
        }
    });
}
