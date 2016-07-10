use x11_dl::xlib::{XCreateWindowEvent, XDestroyWindowEvent};

pub fn on_create_notify(_: XCreateWindowEvent) {}
pub fn on_destroy_notify(_: XDestroyWindowEvent) {}
