
extern crate x11_dl;
extern crate libc;

#[macro_use]
extern crate log;

use x11_dl::xlib;
use std::ptr;

use std::ops::Drop;

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
    let wm = WindowManager::new(&xlib);

}
