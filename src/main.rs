

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
    #[allow(non_upper_case_globals)]

    static ref xlib: Xlib = Xlib::open().expect("Can't open Xlib");
}


// I know it is bad style, but we need a flag to check if another WM is present
#[allow(non_upper_case_globals)]
static mut wm_detected_flag: bool = false;

#[allow(non_upper_case_globals)]
static c_false: i32 = 0;
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
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
