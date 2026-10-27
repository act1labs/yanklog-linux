//! Clipboard capture through GTK for desktops where arboard cannot reach the
//! clipboard: GNOME on Wayland without X11, as in the Flatpak build.
//!
//! Wayland only sends the current selection to the focused client, so a background
//! app learns of a copy in two ways. If yanklog owns the selection, another app
//! copying cancels it, which is reported even without focus. Otherwise the offer
//! yanklog last saw stops being readable once its source is replaced. Either way a
//! nearly transparent window takes focus for a moment, reads the new text and closes.

use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const PASSWORD_MANAGER_HINT: &str = "x-kde-passwordManagerHint";
const READ_TIMEOUT: Duration = Duration::from_secs(2);
const FOCUS_RETRY: Duration = Duration::from_millis(1500);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);
/// Reads fail now and then while the source app is busy; a streak means the offer is gone.
const PROBE_FAILURES_BEFORE_CAPTURE: u32 = 2;

pub struct Hooks {
    /// Whether capture is paused.
    pub is_paused: Box<dyn Fn() -> bool>,
    /// Called with text another app copied. Returns true when it was new.
    pub observe: Box<dyn Fn(&str) -> bool>,
    /// Saves new text to history.
    pub record: Box<dyn Fn(String)>,
}

struct Monitor {
    app: adw::Application,
    clipboard: gtk::gdk::Clipboard,
    hooks: Hooks,
    sentinel: String,
    last_formats: RefCell<String>,
    /// yanklog's own content (a copied entry or the sentinel) is on the clipboard.
    owned: Cell<bool>,
    capturing: Cell<bool>,
    probing: Cell<bool>,
    probe_failures: Cell<u32>,
    /// A capture already failed to read the current offer, so failed reads do not retrigger it.
    captured_offer: Cell<bool>,
    was_paused: Cell<bool>,
}

pub fn start(app: &adw::Application, poll_interval: Duration, hooks: Hooks) {
    let Some(display) = gtk::gdk::Display::default() else {
        eprintln!("No display; clipboard history is off.");
        return;
    };
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let monitor = Rc::new(Monitor {
        app: app.clone(),
        clipboard: display.clipboard(),
        sentinel: format!("\u{200b}yanklog-{}-{stamp}\u{200b}", std::process::id()),
        hooks,
        last_formats: RefCell::new(String::new()),
        owned: Cell::new(false),
        capturing: Cell::new(false),
        probing: Cell::new(false),
        probe_failures: Cell::new(0),
        captured_offer: Cell::new(false),
        was_paused: Cell::new(false),
    });
    monitor.owned.set(monitor.clipboard.is_local());
    *monitor.last_formats.borrow_mut() = formats_text(&monitor.clipboard);

    monitor.clipboard.connect_changed({
        let monitor = Rc::downgrade(&monitor);
        move |_| {
            if let Some(monitor) = monitor.upgrade() {
                monitor.on_changed();
            }
        }
    });
    glib::timeout_add_local(poll_interval, move || {
        monitor.tick();
        glib::ControlFlow::Continue
    });
}

/// Puts text on the clipboard through GTK. The text stays available while yanklog runs.
pub fn set_text(text: &str) -> bool {
    match gtk::gdk::Display::default() {
        Some(display) => {
            display.clipboard().set_text(text);
            true
        }
        None => false,
    }
}

fn formats_text(clipboard: &gtk::gdk::Clipboard) -> String {
    clipboard.formats().to_str().to_string()
}

impl Monitor {
    fn on_changed(self: &Rc<Self>) {
        if self.capturing.get() {
            return;
        }
        if self.clipboard.is_local() {
            self.owned.set(true);
        } else if self.owned.replace(false) {
            // Another app took the selection from yanklog.
            self.capture(true);
        }
    }

    fn tick(self: &Rc<Self>) {
        if self.capturing.get() || self.probing.get() {
            return;
        }
        if (self.hooks.is_paused)() {
            self.was_paused.set(true);
            return;
        }
        if self.was_paused.replace(false) {
            // Note what was copied while paused without saving it.
            self.capture(false);
            return;
        }

        if self.clipboard.is_local() {
            self.owned.set(true);
            return;
        }
        if self.owned.replace(false) {
            self.capture(true);
            return;
        }

        let formats = formats_text(&self.clipboard);
        if formats != *self.last_formats.borrow() {
            self.capture(true);
            return;
        }
        if formats.is_empty() {
            // Nothing to watch; own the clipboard so the next copy cancels it.
            self.clipboard.set_text(&self.sentinel);
            self.owned.set(true);
            return;
        }
        self.probe();
    }

    /// Reads the offer yanklog last saw. A read that keeps failing means it was replaced.
    fn probe(self: &Rc<Self>) {
        self.probing.set(true);
        let monitor = Rc::clone(self);
        glib::spawn_future_local(async move {
            let readable = monitor.read_offer().await;
            monitor.probing.set(false);
            match readable {
                Some(text) => {
                    monitor.probe_failures.set(0);
                    monitor.captured_offer.set(false);
                    // Text that was already there when yanklog started, for example.
                    if let Some(text) = text {
                        monitor.accept(text, true);
                    }
                }
                None => {
                    let failures = monitor.probe_failures.get() + 1;
                    monitor.probe_failures.set(failures);
                    if failures >= PROBE_FAILURES_BEFORE_CAPTURE && !monitor.captured_offer.get() {
                        monitor.capture(true);
                    }
                }
            }
        });
    }

    /// `Some(text)` when the offer can still be read (text only when it has text).
    async fn read_offer(&self) -> Option<Option<String>> {
        let formats = self.clipboard.formats();
        if formats.contain_mime_type("text/plain;charset=utf-8")
            || formats.contain_mime_type("text/plain")
            || formats.contain_mime_type("UTF8_STRING")
        {
            let read = glib::future_with_timeout(READ_TIMEOUT, self.clipboard.read_text_future());
            return match read.await {
                Ok(Ok(text)) => Some(text.map(|text| text.to_string())),
                _ => None,
            };
        }
        let mime_types = formats.mime_types();
        let first = mime_types.first()?;
        let read = glib::future_with_timeout(
            READ_TIMEOUT,
            self.clipboard
                .read_future(&[first.as_str()], glib::Priority::DEFAULT),
        );
        match read.await {
            Ok(Ok((stream, _))) => {
                let _ = stream.close(gtk::gio::Cancellable::NONE);
                Some(None)
            }
            _ => None,
        }
    }

    /// Takes focus long enough to receive the current selection, then reads it.
    fn capture(self: &Rc<Self>, record: bool) {
        if self.capturing.replace(true) {
            return;
        }

        // A yanklog window that already has focus receives the selection directly.
        if self
            .app
            .active_window()
            .is_some_and(|window| window.is_active())
        {
            self.read_and_finish(None, record);
            return;
        }

        let window = gtk::Window::builder()
            .application(&self.app)
            .title("yanklog")
            .decorated(false)
            .resizable(false)
            .default_width(1)
            .default_height(1)
            .build();
        window.set_opacity(0.01);

        let started = Rc::new(Cell::new(false));
        window.connect_is_active_notify({
            let monitor = Rc::clone(self);
            let started = Rc::clone(&started);
            move |window| {
                if window.is_active() && !started.replace(true) {
                    monitor.read_and_finish(Some(window.clone()), record);
                }
            }
        });
        glib::timeout_add_local_once(FOCUS_RETRY, {
            let window = window.downgrade();
            let started = Rc::clone(&started);
            move || {
                if let Some(window) = window.upgrade() {
                    if !started.get() {
                        window.present();
                    }
                }
            }
        });
        glib::timeout_add_local_once(CAPTURE_TIMEOUT, {
            let monitor = Rc::clone(self);
            let window = window.downgrade();
            let started = Rc::clone(&started);
            move || {
                if !started.get() {
                    eprintln!("Clipboard capture did not get focus; skipped this copy.");
                    if let Some(window) = window.upgrade() {
                        window.close();
                    }
                    monitor.finish(false);
                }
            }
        });
        window.present();
    }

    fn read_and_finish(self: &Rc<Self>, window: Option<gtk::Window>, record: bool) {
        let monitor = Rc::clone(self);
        glib::spawn_future_local(async move {
            let secret = monitor
                .clipboard
                .formats()
                .contain_mime_type(PASSWORD_MANAGER_HINT);
            let read = monitor.read_offer().await;
            if let Some(Some(text)) = &read {
                if secret {
                    (monitor.hooks.observe)(text);
                } else {
                    monitor.accept(text.clone(), record);
                }
            }
            if let Some(window) = window {
                window.close();
            }
            monitor.finish(read.is_some());
        });
    }

    fn accept(&self, text: String, record: bool) {
        if text == self.sentinel {
            return;
        }
        if (self.hooks.observe)(&text) && record && !(self.hooks.is_paused)() {
            (self.hooks.record)(text);
        }
    }

    /// `read_ok` is false when the selection could not be read even with focus, for
    /// example because its owner quit; failed reads then do not trigger it again.
    fn finish(&self, read_ok: bool) {
        *self.last_formats.borrow_mut() = formats_text(&self.clipboard);
        self.probe_failures.set(0);
        self.captured_offer.set(!read_ok);
        self.owned.set(self.clipboard.is_local());
        self.capturing.set(false);
    }
}
