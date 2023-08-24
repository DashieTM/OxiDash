pub mod imp;

use glib::Object;
use gtk::{gio, glib};

glib::wrapper! {
    pub struct NotiBox(ObjectSubclass<imp::NotiBox>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget;
}

impl NotiBox {
    pub fn new(orientation: gtk::Orientation, spacing: i32) -> Self {
        Object::builder()
            .property("orientation", orientation)
            .property("spacing", spacing)
            .build()
    }
}

impl Default for NotiBox {
    fn default() -> Self {
        Object::builder().build()
    }
}
