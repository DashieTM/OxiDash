use dbus::blocking::Connection;
use gtk::glib::Sender;

use crate::{Notification, Urgency};

pub fn run(sender: Sender<Notification>) {
    let c = Connection::new_session().unwrap();
    c.request_name("org.freedesktop.NotificationCenter", false, true, false)
        .unwrap();
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("org.freedesktop.NotificationCenter", |c| {
        c.method(
            "Notify",
            (
                "app_name",
                "replaces_id",
                "app_icon",
                "summary",
                "body",
                "actions",
                "expire_timeout",
                "urgency",
                "image_path",
                "progress",
            ),
            ("reply",),
            move |_,
                  _,
                  (
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions,
                expire_timeout,
                urgency,
                image_path,
                progress,
            ): (
                String,
                u32,
                String,
                String,
                String,
                Vec<String>,
                i32,
                i32,
                String,
                i32,
            )| {
                let notification = Notification {
                    app_name,
                    replaces_id,
                    app_icon,
                    summary,
                    body,
                    actions,
                    expire_timeout,
                    urgency: Urgency::from_i32(urgency).unwrap_or_else(|_| Urgency::Low),
                    image_path,
                    progress,
                };
                sender
                    .send(notification)
                    .expect("Failed to send notification.");
                Ok(("ok",))
            },
        );
    });
    cr.insert("/org/freedesktop/NotificationCenter", &[token], ());
    cr.serve(&c).unwrap();
}
