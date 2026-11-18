//! Download handling.
//!
//! WebView2 on Windows ships its own download flyout. WebKitGTK does not, so
//! this module saves files to the user's Downloads folder and shows a small
//! bar at the bottom of the window with progress and Open / Show in Folder.

use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{gio, glib};
use webkit6::Download;

#[derive(Clone)]
pub struct DownloadBar {
    revealer: gtk::Revealer,
    list: gtk::Box,
}

impl DownloadBar {
    pub fn new() -> Self {
        let list = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .hexpand(true)
            .child(&list)
            .build();

        let close = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Hide downloads")
            .valign(gtk::Align::Center)
            .css_classes(["flat"])
            .build();

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        bar.set_margin_top(6);
        bar.set_margin_bottom(6);
        bar.set_margin_start(6);
        bar.set_margin_end(6);
        bar.append(&scroller);
        bar.append(&close);

        let frame = gtk::Box::new(gtk::Orientation::Vertical, 0);
        frame.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        frame.append(&bar);

        let revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideUp)
            .child(&frame)
            .reveal_child(false)
            .build();

        close.connect_clicked(glib::clone!(
            #[weak]
            revealer,
            #[weak]
            list,
            move |_| {
                revealer.set_reveal_child(false);
                // Drop finished/failed entries; keep in-progress ones.
                let mut child = list.first_child();
                while let Some(c) = child {
                    child = c.next_sibling();
                    if c.has_css_class("done") {
                        list.remove(&c);
                    }
                }
            }
        ));

        Self { revealer, list }
    }

    pub fn widget(&self) -> &gtk::Revealer {
        &self.revealer
    }

    pub fn track(&self, window: &gtk::ApplicationWindow, download: &Download) {
        let name = gtk::Label::builder()
            .label("Starting download…")
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(32)
            .css_classes(["heading"])
            .build();
        let status = gtk::Label::builder()
            .xalign(0.0)
            .css_classes(["dim-label", "caption"])
            .build();
        let progress = gtk::ProgressBar::new();

        let open = gtk::Button::builder().label("Open").visible(false).build();
        let folder = gtk::Button::builder()
            .label("Show in Folder")
            .visible(false)
            .build();
        let cancel = gtk::Button::builder().label("Cancel").build();

        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        buttons.append(&open);
        buttons.append(&folder);
        buttons.append(&cancel);

        let item = gtk::Box::new(gtk::Orientation::Vertical, 4);
        item.set_width_request(260);
        item.append(&name);
        item.append(&progress);
        item.append(&status);
        item.append(&buttons);

        self.list.prepend(&item);
        self.revealer.set_reveal_child(true);

        // Pick a non-clashing file name in ~/Downloads.
        download.connect_decide_destination(glib::clone!(
            #[weak]
            name,
            #[upgrade_or]
            false,
            move |dl, suggested| {
                let path = unique_destination(&downloads_dir(), suggested);
                name.set_label(
                    &path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| suggested.to_owned()),
                );
                name.set_tooltip_text(path.to_str());
                if let Some(p) = path.to_str() {
                    dl.set_destination(p);
                }
                true
            }
        ));

        download.connect_received_data(glib::clone!(
            #[weak]
            progress,
            #[weak]
            status,
            move |dl, _| {
                let fraction = dl.estimated_progress();
                progress.set_fraction(fraction);
                let received = glib::format_size(dl.received_data_length());
                let total = dl
                    .response()
                    .map(|r| r.content_length())
                    .filter(|len| *len > 0);
                status.set_label(&match total {
                    Some(t) => format!("{received} of {}", glib::format_size(t)),
                    None => received.to_string(),
                });
            }
        ));

        download.connect_finished(glib::clone!(
            #[weak]
            item,
            #[weak]
            progress,
            #[weak]
            status,
            #[weak]
            open,
            #[weak]
            folder,
            #[weak]
            cancel,
            move |dl| {
                item.add_css_class("done");
                cancel.set_visible(false);
                // "finished" is also emitted after a failure; only mark success
                // if we didn't already show an error.
                if status.has_css_class("error") {
                    return;
                }
                progress.set_fraction(1.0);
                let size = glib::format_size(dl.received_data_length());
                status.set_label(&format!("Completed — {size}"));
                if dl.destination().is_some() {
                    open.set_visible(true);
                    folder.set_visible(true);
                }
            }
        ));

        download.connect_failed(glib::clone!(
            #[weak]
            item,
            #[weak]
            status,
            #[weak]
            cancel,
            move |_, error| {
                item.add_css_class("done");
                cancel.set_visible(false);
                status.add_css_class("error");
                status.remove_css_class("dim-label");
                if error.matches(webkit6::DownloadError::CancelledByUser) {
                    status.set_label("Cancelled");
                } else {
                    status.set_label(&format!("Failed: {}", error.message()));
                }
            }
        ));

        cancel.connect_clicked(glib::clone!(
            #[weak]
            download,
            move |_| download.cancel()
        ));

        let win = window.downgrade();
        let dl = download.clone();
        open.connect_clicked(move |_| {
            if let Some(file) = destination_file(&dl) {
                gtk::FileLauncher::new(Some(&file)).launch(
                    win.upgrade().as_ref(),
                    gio::Cancellable::NONE,
                    |_| {},
                );
            }
        });

        let win = window.downgrade();
        let dl = download.clone();
        folder.connect_clicked(move |_| {
            if let Some(file) = destination_file(&dl) {
                gtk::FileLauncher::new(Some(&file)).open_containing_folder(
                    win.upgrade().as_ref(),
                    gio::Cancellable::NONE,
                    |_| {},
                );
            }
        });
    }
}

fn destination_file(download: &Download) -> Option<gio::File> {
    download.destination().map(|d| gio::File::for_path(d.as_str()))
}

fn downloads_dir() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Downloads).unwrap_or_else(glib::home_dir)
}

/// `dir/name`, or `dir/name (1).ext`, `dir/name (2).ext`, … if taken.
fn unique_destination(dir: &Path, suggested: &str) -> PathBuf {
    // Never trust a server-provided name to stay inside `dir`.
    let file_name = Path::new(suggested)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty() && n != "." && n != "..")
        .unwrap_or_else(|| "download".to_owned());

    let candidate = dir.join(&file_name);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(&file_name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.clone());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    (1..)
        .map(|i| dir.join(format!("{stem} ({i}){ext}")))
        .find(|p| !p.exists())
        .expect("infinite iterator")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names() {
        let dir = std::env::temp_dir().join(format!("ldwp-dl-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(unique_destination(&dir, "a.zip"), dir.join("a.zip"));
        std::fs::write(dir.join("a.zip"), b"").unwrap();
        assert_eq!(unique_destination(&dir, "a.zip"), dir.join("a (1).zip"));
        assert_eq!(unique_destination(&dir, "../../etc/passwd"), dir.join("passwd"));
        assert_eq!(unique_destination(&dir, ".."), dir.join("download"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
