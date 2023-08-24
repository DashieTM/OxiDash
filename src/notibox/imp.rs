use std::cell::{RefCell, Cell};

use gtk::subclass::prelude::*;
use gtk::{glib, Image, Label, ProgressBar};

#[derive(Default)]
pub struct NotiBox {
    pub progbar: RefCell<ProgressBar>,
    pub has_progbar: Cell<bool>,
    pub body: RefCell<Label>,
    pub has_body: Cell<bool>,
    pub summary: RefCell<Label>,
    pub has_summary: Cell<bool>,
    pub image: RefCell<Image>,
    pub has_image: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for NotiBox {
    const NAME: &'static str = "NotiBox";
    type Type = super::NotiBox;
    type ParentType = gtk::Box;
}

impl ObjectImpl for NotiBox {}

impl WidgetImpl for NotiBox {}

impl BoxImpl for NotiBox {}
