use x11_dl::xlib::{XCreateWindowEvent, XDestroyWindowEvent, XReparentEvent, XMapEvent,
                   XUnmapEvent, XConfigureEvent, XMapRequestEvent, XConfigureRequestEvent,
                   XButtonEvent, XMotionEvent, XKeyEvent};

pub fn on_create_notify(_: XCreateWindowEvent) {}
pub fn on_destroy_notify(_: XDestroyWindowEvent) {}
pub fn on_reparent_notify(_: XReparentEvent) {}
pub fn on_map_notify(_: XMapEvent) {}
pub fn on_unmap_notify(_: XUnmapEvent) {}
pub fn on_configure_notify(_: XConfigureEvent) {}
pub fn on_map_request(_: XMapRequestEvent) {}
pub fn on_configure_request_event(_: XConfigureRequestEvent) {}
pub fn on_button_press(_: XButtonEvent) {}
pub fn on_button_release(_: XButtonEvent) {}
pub fn on_motion_notify(_: XMotionEvent) {}
pub fn on_key_press(_: XKeyEvent) {}
pub fn on_key_release(_: XKeyEvent) {}
