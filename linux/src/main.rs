//! LDWP Collective for Desktop — Linux edition.
//!
//! A GTK 4 / WebKitGTK reimplementation of the Windows (WinForms + WebView2)
//! application: a single window that hosts the LDWP Collective website.

mod downloads;
mod window;

use gtk::prelude::*;
use gtk::{gio, glib};

pub const APP_ID: &str = "org.ldwp.CollectiveDesktop";
pub const APP_TITLE: &str = "LDWP Collective for Desktop";
pub const WEBSITE_URL: &str = "https://ldwp.org";

fn main() -> glib::ExitCode {
    glib::set_application_name(APP_TITLE);
    glib::set_prgname(Some(APP_ID));

    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::default())
        .build();

    // GTK is initialized by the time `startup` fires.
    app.connect_startup(|app| {
        gtk::Window::set_default_icon_name(APP_ID);
        window::register_accels(app);
    });

    app.connect_activate(|app| {
        // Single instance: launching the app again focuses the existing window.
        if let Some(existing) = app.active_window() {
            existing.present();
            return;
        }
        window::build(app).present();
    });

    app.run()
}
