use dbus::blocking::Connection;
use gtk::glib::Sender;

use crate::{ImageData, Notification, Urgency};

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
                "data",
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
                raw_data,
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
                (i32, i32, i32, bool, i32, i32, Vec<u8>),
            )| {
                let image_data = ImageData {
                    width: raw_data.0,
                    height: raw_data.1,
                    rowstride: raw_data.2,
                    has_alpha: raw_data.3,
                    bits_per_sample: raw_data.4,
                    channels: raw_data.5,
                    data: raw_data.6,
                };
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
                    image_data,
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
