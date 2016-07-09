
extern crate x11_dl;
extern crate libc;

#[macro_use]
extern crate log;

use x11_dl::xlib;
use std::ptr;

use std::ops::Drop;

// I know it is bad style, but we need a flag to check if another WM is present
static mut wm_detected_flag: bool = false;

static c_false: i32 = 0;
static c_true: i32 = 1;

mod logging {
    extern crate log;
    use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};

    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &LogMetadata) -> bool {
            metadata.level() <= LogLevel::Info
        }

        fn log(&self, record: &LogRecord) {
            if self.enabled(record.metadata()) {
                println!("{} - {}", record.level(), record.args());
            }
        }
    }

    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Info);
            Box::new(SimpleLogger)
        })
    }
}

unsafe extern "C" fn wm_detected_handler(display: *mut xlib::Display,
                                         err: *mut xlib::XErrorEvent)
                                         -> i32 {
    let aerr = *err;
    // BadAccess aka Error code 10 means another WM is present
    if aerr.error_code == xlib::BadAccess {
        wm_detected_flag = true
    }
    0
}
unsafe extern "C" fn wm_error_handler(display: *mut xlib::Display,
                                      err: *mut xlib::XErrorEvent)
                                      -> i32 {
    let aerr = *err;
    // Print the Error code
    error!("Error Catched {:?}", aerr.error_code);
    0
}


struct WindowManager {
    display: *const xlib::Display,
    root_window: *const xlib::Window,
    xlib: *const xlib::Xlib,
}

impl WindowManager {
    fn new(xlib: &xlib::Xlib) -> Self {
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
            let root: *const xlib::Window = &((xlib.XDefaultRootWindow)(display));


            WindowManager {
                display: display,
                root_window: root,
                xlib: xlib,
            }
        }

    }
    fn run(&mut self) -> () {
        let xlib = {
            let lib = self.xlib;
            unsafe { &*lib }
        };


        unsafe {
            // Set a Error Handler just for finding another WM
            {
                (xlib.XSetErrorHandler)(Some(wm_detected_handler));
                let _ = (xlib.XSelectInput)(self.display as *mut _,
                                            *self.root_window,
                                            xlib::SubstructureRedirectMask |
                                            xlib::SubstructureNotifyMask);
                (xlib.XSync)(self.display as *mut _, c_false);

                if wm_detected_flag {
                    error!("Another WM is running.!");
                    return;
                }
            }
            // Set the real Error handler
            (xlib.XSetErrorHandler)(Some(wm_error_handler));

            loop {

                let e: *mut xlib::XEvent = ptr::null_mut();
                (xlib.XNextEvent)(self.display as *mut _, e);
            }
        }

    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        let lib = self.xlib;
        info!("Dropping the WM struct");
        unsafe {
            let ref rlib = *lib;
            (rlib.XDestroyWindow)(self.display as *mut _, *(self.root_window));
            (rlib.XCloseDisplay)(self.display as *mut _);
        }

    }
}
fn main() {
    let _ = logging::init();
    info!("Starting WM");
    let xlib: xlib::Xlib;
    xlib = xlib::Xlib::open().expect("Can't open Xlib");
    let mut wm = WindowManager::new(&xlib);
    wm.run();
}
