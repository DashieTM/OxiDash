use std::cell::RefCell;

use gtk::subclass::prelude::*;
use gtk::{glib, Image, Label, ProgressBar};

#[derive(Default)]
pub struct NotiBox {
    pub progbar: RefCell<ProgressBar>,
    pub body: RefCell<Label>,
    pub summary: RefCell<Label>,
    pub image: RefCell<Image>,
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
