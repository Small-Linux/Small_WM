// Xlib uses a lot of non upper case constants. This is just to ignore all these warnings
#![allow(non_upper_case_globals)]

extern crate x11_dl;
extern crate libc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use std::ptr;

use std::ops::Drop;

use x11_dl::xlib::*;

lazy_static! {

    static ref xlib: Xlib = Xlib::open().expect("Can't open Xlib");
}


// I know it is bad style, but we need a flag to check if another WM is present
static mut wm_detected_flag: bool = false;

static c_false: i32 = 0;
static c_true: i32 = 1;

pub mod logging;

pub mod event_handler;


unsafe extern "C" fn wm_detected_handler(_: *mut Display, err: *mut XErrorEvent) -> i32 {
    // BadAccess aka Error code 10 means another WM is present
    if (*err).error_code == BadAccess {
        wm_detected_flag = true
    }
    0
}
unsafe extern "C" fn wm_error_handler(_: *mut Display, err: *mut XErrorEvent) -> i32 {
    // Print the Error code
    error!("Error Catched {:?}", (*err).error_code);
    0
}


struct WindowManager {
    display: *const Display,
    root_window: *const Window,
}

impl WindowManager {
    fn new() -> Self {
        unsafe {

            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                error!("Cannot connect to display server: {:?}",
                       (xlib.XDisplayName)(ptr::null()));
                panic!();
            }
            info!("Connected to Display Server: {:?}",
                  (xlib.XDisplayName)(ptr::null()) as *const char);

            info!("Makeing the display the root window");
            let root: *const Window = &((xlib.XDefaultRootWindow)(display));


            WindowManager {
                display: display,
                root_window: root,
            }
        }

    }
    fn run(&mut self) -> () {

        unsafe {
            // Set a Error Handler just for finding another WM
            {
                (xlib.XSetErrorHandler)(Some(wm_detected_handler));
                let _ = (xlib.XSelectInput)(self.display as *mut _,
                                            *self.root_window,
                                            SubstructureRedirectMask | SubstructureNotifyMask);
                (xlib.XSync)(self.display as *mut _, c_false);

                if wm_detected_flag {
                    error!("Another WM is running.!");
                    return;
                }
            }
            // Set the real Error handler
            (xlib.XSetErrorHandler)(Some(wm_error_handler));

            loop {
                let e: *mut XEvent = ptr::null_mut();
                (xlib.XNextEvent)(self.display as *mut _, e);
                info!("Received Event: {:?}", e);

                match (*e).get_type() {
                    CreateNotify => {
                        let event: XCreateWindowEvent = From::from(*e);
                        event_handler::on_create_notify(event);
                    }

                    DestroyNotify => {
                        let event: XDestroyWindowEvent = From::from(*e);
                        event_handler::on_destroy_notify(event);
                    }
                    ReparentNotify => {
                        let event: XReparentEvent = From::from(*e);
                        event_handler::on_reparent_notify(event);
                    }
                    MapNotify => {
                        let event: XMapEvent = From::from(*e);
                        event_handler::on_map_notify(event);
                    }
                    UnmapNotify => {
                        let event: XUnmapEvent = From::from(*e);
                        event_handler::on_unmap_notify(event);
                    }
                    ConfigureNotify => {
                        let event: XConfigureEvent = From::from(*e);
                        event_handler::on_configure_notify(event);
                    }
                    MapRequest => {
                        let event: XMapRequestEvent = From::from(*e);
                        event_handler::on_map_request(event);
                    }
                    ConfigureRequest => {
                        let event: XConfigureRequestEvent = From::from(*e);
                        event_handler::on_configure_request_event(event);
                    }
                    ButtonPress => {
                        let event: XButtonEvent = From::from(*e);
                        event_handler::on_button_press(event);
                    }
                    ButtonRelease => {
                        let event: XButtonEvent = From::from(*e);
                        event_handler::on_button_release(event);
                    }
                    MotionNotify => {
                        let event: XMotionEvent = From::from(*e);
                        event_handler::on_motion_notify(event);
                    }
                    KeyPress => {
                        let event: XKeyEvent = From::from(*e);
                        event_handler::on_key_press(event);
                    }
                    KeyRelease => {
                        let event: XKeyEvent = From::from(*e);
                        event_handler::on_key_release(event);
                    }
                    _ => (),
                }

            }
        }

    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        info!("Dropping the WM struct");
        unsafe {
            (xlib.XDestroyWindow)(self.display as *mut _, *(self.root_window));
            (xlib.XCloseDisplay)(self.display as *mut _);
        }

    }
}
fn main() {
    let _ = logging::init();
    info!("Starting WM");
    let mut wm = WindowManager::new();
    wm.run();
}
