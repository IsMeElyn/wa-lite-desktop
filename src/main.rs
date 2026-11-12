// Sembunyikan jendela konsol pada build release di Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder};

const HOME: &str = "https://web.whatsapp.com/";

// WhatsApp Web menolak browser yang tidak dikenal, jadi di Linux (WebKitGTK)
// kita memakai User-Agent Chrome.
#[cfg(target_os = "linux")]
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
Chrome/124.0.0.0 Safari/537.36";

/// Hanya web.whatsapp.com yang boleh dimuat di jendela ini.
fn allowed(url: &str) -> bool {
    url.starts_with("https://web.whatsapp.com")
        || url.starts_with("blob:https://web.whatsapp.com")
        || url == "about:blank"
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WhatsApp")
        .with_inner_size(LogicalSize::new(1100.0, 760.0))
        .with_min_inner_size(LogicalSize::new(420.0, 520.0))
        .build(&event_loop)
        .expect("gagal membuat jendela");

    let mut context = WebContext::new(Some(data_dir()));

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
        .with_url(HOME)
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

    #[cfg(target_os = "linux")]
    let builder = builder.with_user_agent(UA);

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

    event_loop.run(move |event, _, control_flow| {
        let _keep_alive = (&webview, &context, &window);
        *control_flow = ControlFlow::Wait; // tidak polling = CPU idle ~0%
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
