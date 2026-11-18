//! Main window: equivalent of `Form1` in the Windows version.

use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use webkit6::prelude::*;
use webkit6::{
    CookiePersistentStorage, LoadEvent, NavigationPolicyDecision, NetworkError, NetworkSession,
    PolicyDecisionType, PolicyError, WebView,
};

use crate::downloads::DownloadBar;
use crate::{APP_ID, APP_TITLE, WEBSITE_URL};

const ZOOM_STEP: f64 = 0.1;
const ZOOM_MIN: f64 = 0.25;
const ZOOM_MAX: f64 = 5.0;

/// Keyboard shortcuts mirroring the ones WebView2 provides on Windows.
pub fn register_accels(app: &gtk::Application) {
    app.set_accels_for_action("win.reload", &["F5", "<Control>r"]);
    app.set_accels_for_action("win.reload-hard", &["<Control>F5", "<Control><Shift>r"]);
    app.set_accels_for_action("win.back", &["<Alt>Left", "Back"]);
    app.set_accels_for_action("win.forward", &["<Alt>Right", "Forward"]);
    app.set_accels_for_action("win.home", &["<Alt>Home"]);
    app.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal", "<Control>KP_Add"]);
    app.set_accels_for_action("win.zoom-out", &["<Control>minus", "<Control>KP_Subtract"]);
    app.set_accels_for_action("win.zoom-reset", &["<Control>0", "<Control>KP_0"]);
    app.set_accels_for_action("win.fullscreen", &["F11"]);
    app.set_accels_for_action("win.inspector", &["F12", "<Control><Shift>i"]);
}

/// Persistent storage for cookies, local storage, IndexedDB, cache, etc.
/// (WebView2 keeps the equivalent in its user data folder.)
fn network_session() -> NetworkSession {
    let data_dir: PathBuf = glib::user_data_dir().join(APP_ID).join("webkit");
    let cache_dir: PathBuf = glib::user_cache_dir().join(APP_ID).join("webkit");
    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(&cache_dir);

    let session = NetworkSession::new(data_dir.to_str(), cache_dir.to_str());
    if let Some(cookies) = session.cookie_manager() {
        let cookie_db = data_dir.join("cookies.sqlite");
        if let Some(path) = cookie_db.to_str() {
            cookies.set_persistent_storage(path, CookiePersistentStorage::Sqlite);
        }
    }
    session
}

/// `true` if `uri` points at ldwp.org or one of its subdomains.
fn is_ldwp_uri(uri: &str) -> bool {
    glib::Uri::parse(uri, glib::UriFlags::NONE)
        .ok()
        .and_then(|u| u.host())
        .map(|host| {
            let host = host.to_ascii_lowercase();
            host == "ldwp.org" || host.ends_with(".ldwp.org")
        })
        .unwrap_or(false)
}

pub fn build(app: &gtk::Application) -> gtk::ApplicationWindow {
    let session = network_session();

    let webview = WebView::builder().network_session(&session).build();
    webview.set_background_color(&gdk::RGBA::WHITE);
    webview.set_vexpand(true);
    webview.set_hexpand(true);
    if let Some(settings) = WebViewExt::settings(&webview) {
        settings.set_enable_developer_extras(true);
        settings.set_javascript_can_open_windows_automatically(true);
        settings.set_enable_back_forward_navigation_gestures(true);
    }

    let downloads = DownloadBar::new();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&webview);
    content.append(downloads.widget());

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .icon_name(APP_ID)
        .default_width(1200)
        .default_height(800)
        .child(&content)
        .build();

    // Downloads (WebView2 shows its own download flyout; WebKitGTK needs us to).
    {
        let downloads = downloads.clone();
        let window = window.downgrade();
        session.connect_download_started(move |_, download| {
            if let Some(window) = window.upgrade() {
                downloads.track(&window, download);
            }
        });
    }

    // target="_blank" / window.open(): stay in-app for LDWP pages, hand
    // everything else to the user's default browser.
    {
        let window = window.downgrade();
        webview.connect_decide_policy(move |view, decision, kind| {
            if kind != PolicyDecisionType::NewWindowAction {
                return false;
            }
            let Some(uri) = decision
                .downcast_ref::<NavigationPolicyDecision>()
                .and_then(|d| d.navigation_action())
                .and_then(|a| a.request())
                .and_then(|r| r.uri())
            else {
                return false;
            };
            decision.ignore();
            if is_ldwp_uri(&uri) || uri.starts_with("about:") {
                view.load_uri(&uri);
            } else {
                gtk::UriLauncher::new(&uri).launch(
                    window.upgrade().as_ref(),
                    gio::Cancellable::NONE,
                    |_| {},
                );
            }
            true
        });
    }

    // Equivalent of the "Unable to load LDWP" MessageBox on Windows.
    {
        let window = window.downgrade();
        webview.connect_load_failed(move |view, _event: LoadEvent, failing_uri, error| {
            if error.matches(NetworkError::Cancelled)
                || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange)
            {
                return false;
            }
            if let Some(window) = window.upgrade() {
                show_load_error(&window, view, failing_uri, error);
            }
            false // still let WebKit render its built-in error page
        });
    }

    // Keep the window title in sync with the page, like a browser would,
    // while always identifying the app.
    webview.connect_title_notify(glib::clone!(
        #[weak]
        window,
        move |view| {
            match view.title().filter(|t| !t.is_empty()) {
                Some(t) => window.set_title(Some(&format!("{t} — {APP_TITLE}"))),
                None => window.set_title(Some(APP_TITLE)),
            }
        }
    ));

    // HTML5 fullscreen requests (videos etc.).
    webview.connect_enter_fullscreen(glib::clone!(
        #[weak]
        window,
        #[upgrade_or]
        false,
        move |_| {
            window.fullscreen();
            false
        }
    ));
    webview.connect_leave_fullscreen(glib::clone!(
        #[weak]
        window,
        #[upgrade_or]
        false,
        move |_| {
            window.unfullscreen();
            false
        }
    ));

    install_actions(&window, &webview);

    webview.load_uri(WEBSITE_URL);
    window
}

fn install_actions(window: &gtk::ApplicationWindow, webview: &WebView) {
    let add = |name: &str, f: Box<dyn Fn(&WebView, &gtk::ApplicationWindow)>| {
        let action = gio::SimpleAction::new(name, None);
        let view = webview.downgrade();
        let win = window.downgrade();
        action.connect_activate(move |_, _| {
            if let (Some(view), Some(win)) = (view.upgrade(), win.upgrade()) {
                f(&view, &win);
            }
        });
        window.add_action(&action);
    };

    add("reload", Box::new(|v, _| v.reload()));
    add("reload-hard", Box::new(|v, _| v.reload_bypass_cache()));
    add("back", Box::new(|v, _| v.go_back()));
    add("forward", Box::new(|v, _| v.go_forward()));
    add("home", Box::new(|v, _| v.load_uri(WEBSITE_URL)));
    add(
        "zoom-in",
        Box::new(|v, _| v.set_zoom_level((v.zoom_level() + ZOOM_STEP).min(ZOOM_MAX))),
    );
    add(
        "zoom-out",
        Box::new(|v, _| v.set_zoom_level((v.zoom_level() - ZOOM_STEP).max(ZOOM_MIN))),
    );
    add("zoom-reset", Box::new(|v, _| v.set_zoom_level(1.0)));
    add(
        "fullscreen",
        Box::new(|_, w| {
            if w.is_fullscreen() {
                w.unfullscreen()
            } else {
                w.fullscreen()
            }
        }),
    );
    add(
        "inspector",
        Box::new(|v, _| {
            if let Some(inspector) = v.inspector() {
                inspector.show();
            }
        }),
    );
}

fn show_load_error(
    window: &gtk::ApplicationWindow,
    webview: &WebView,
    failing_uri: &str,
    error: &glib::Error,
) {
    let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message("Unable to load LDWP")
        .detail(format!(
            "The LDWP website could not be loaded. Check your internet connection and try again.\n\n{failing_uri}\n\n{}",
            error.message()
        ))
        .buttons(["Close", "Retry"])
        .cancel_button(0)
        .default_button(1)
        .build();

    let view = webview.downgrade();
    let uri = failing_uri.to_owned();
    dialog.choose(Some(window), gio::Cancellable::NONE, move |res| {
        if let (Ok(1), Some(view)) = (res, view.upgrade()) {
            view.load_uri(&uri);
        }
    });
}
