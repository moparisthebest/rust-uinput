#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use crate::*;
use std::env;
use std::process::exit;

use getopts::Options;
use std::fs::File;

pub mod codes;
use codes::*;

use lazy_static::lazy_static;
use std::sync::Mutex;
use core_graphics::event::CGKeyCode;
use core_graphics::event::*;
use core_foundation_sys::*;
use core_foundation_sys::base::*;
use core_foundation_sys::runloop::*;

/*
extern {
    /// Return the type identifier for the opaque type `CGEventRef'.
    //fn CGEventGetTypeID() -> CFTypeID;
    pub fn CFRunLoopRun();
}
*/

type MacOSKeyMaps = KeyMaps<Device, CGKeyCode, InputEvent, Option<CGEventRef>>;

// this is used for identifying the fake keypresses we insert, so we don't process them in an infinite loop
//const FAKE_EXTRA_INFO: ULONG_PTR = 332;

//const BLOCK_KEY: *const CGEventRef = std::ptr::null();
//const BLOCK_KEY: *mut CGEventRef = std::ptr::null_mut();

pub struct InputEvent {
     event_type: CGEventType,
     event: CGEventRef,
}

impl KeyEvent<CGKeyCode> for InputEvent {
    fn code(&self) -> CGKeyCode {
        //1
        //self.event.to_owned().get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as CGKeyCode
        unsafe { CGEventGetIntegerValueField(self.event, kCGKeyboardEventKeycode) }
    }

    fn value(&self) -> KeyState {
        match self.event_type {
            kCGEventKeyDown => KeyState::DOWN,
            kCGEventKeyUp => KeyState::UP,
            kCGEventTapDisabledByTimeout => {
                println!("Quartz event tap disabled because of timeout; attempting to reregister.");
                //register_listener(channel);
                KeyState::OTHER
            },
            _ => {
                println!("Received unknown EventType: {}", self.event_type);
                KeyState::OTHER
            },
        }
    }
}

pub enum DeviceRet {
    EVENT(CGEventRef),
    NULL,
}

pub struct Device;

impl Keyboard<CGKeyCode, InputEvent, Option<CGEventRef>> for Device {
    fn send(&self, event: &mut InputEvent) -> Result<Option<CGEventRef>> {
        Ok(Some(event.event))
    }

    fn send_mod_code(&self, code: CGKeyCode, event: &mut InputEvent) -> Result<Option<CGEventRef>> {
        // event.value should only ever be UP/DOWN when this method is called
        Ok(None)
    }

    fn send_mod_code_value(&self, code: CGKeyCode, up_not_down: bool, _event: &mut InputEvent) -> Result<Option<CGEventRef>> {
        Ok(None)
    }

    fn synchronize(&self) -> Result<Option<CGEventRef>> {
        // no-op here
        Ok(None)
    }

    fn left_shift_code(&self) -> CGKeyCode {
        KEY_LEFTSHIFT
    }

    fn right_shift_code(&self) -> CGKeyCode {
        KEY_RIGHTSHIFT
    }

    fn caps_lock_code(&self) -> CGKeyCode {
        KEY_CAPSLOCK
    }

    fn block_key(&self) -> Result<Option<CGEventRef>> {
        Ok(None)
    }
}

unsafe impl Send for MacOSKeyMaps {
    // windows promises us keybd_proc will only be called by a single thread at a time
    // but rust makes us wrap in mutex anyway, so we are extra safe...
}

const DEVICE: Device = Device;

/*
lazy_static! {
static ref KEY_MAPPER: Mutex<MacOSKeyMaps> = {

    let config = parse_args();
    //println!("Config: {:?}", config);

    let key_map = key_map();
    //println!("key_map: {:?}", key_map);

    println!("chosen config file: {}", config.config_file);

    Mutex::new(MacOSKeyMaps::from_cfg(&key_map, &config.config_file))
};
}
*/


pub fn main_res() -> Result<()> {
    // this is just to cause the lazy_static init to run first, so if -h or -v is wanted, we do that
    // and exit immediately... todo: how to avoid mutex/lazy_static entirely???
    //let _ = KEY_MAPPER.lock().unwrap();

    let config = parse_args();
    //println!("Config: {:?}", config);

    let key_map = key_map();
    //println!("key_map: {:?}", key_map);

    println!("chosen config file: {}", config.config_file);

    let key_maps = MacOSKeyMaps::from_cfg(&key_map, &config.config_file);

    let mask = CGEventMaskBit(kCGEventKeyDown)
        | CGEventMaskBit(kCGEventKeyUp);

    unsafe {
        let options = 0;

        // Create the event tap
        let event_tap = CGEventTapCreate(
            kCGSessionEventTap,
            kCGHeadInsertEventTap,
            options,
            mask,
            callback,
            &key_maps,
        );
        assert!(!event_tap.is_null(),
                "Unable to create event tap. Please make sure you have the correct permissions");
        println!("Created event tap...");

        let allocator = kCFAllocatorDefault;
        let current_event_loop = CFRunLoopGetCurrent();
        let mode = kCFRunLoopCommonModes;

        // Create Run Loop Source
        let run_loop_source = CFMachPortCreateRunLoopSource(allocator, event_tap, 0);

        // Add Run Loop Source to the current event loop
        CFRunLoopAddSource(current_event_loop, run_loop_source, mode);

        // Enable the tap
        CGEventTapEnable(event_tap, true);

        CFRunLoopRun();
    }

    unsafe {
        /*
        CFMachPortRef      eventTap;
        CGEventMask        eventMask;
        CFRunLoopSourceRef runLoopSource;


        // Create an event tap. We are interested in key presses.
        let eventMask = ((1 << kCGEventKeyDown) | (1 << kCGEventKeyUp));
        let eventTap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, 0,
                                    eventMask, myCGEventCallback, NULL);

        */
        /*
        if (!eventTap) {
            fprintf(stderr, "failed to create event tap\n");
            exit(1);
        }

        // Create a run loop source.
        let runLoopSource = CFMachPortCreateRunLoopSource(
            kCFAllocatorDefault, eventTap, 0);

        // Add to the current run loop.
        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource,
                           kCFRunLoopCommonModes);

        // Enable the event tap.
        CGEventTapEnable(eventTap, true);

        // Set it all running.
        println!("woo1");
        CFRunLoopRun();
        println!("woo2");
        */
    }
    
    //std::thread::sleep(std::time::Duration::from_millis(400000));
    
    Ok(())
}

#[derive(Debug)]
struct Config {
    config_file: String
}

impl Config {
    fn new(config_file: String) -> Self {
        Config { config_file: config_file }
    }
}

fn get_env_push(key: &str, to_push: &str, vec: &mut Vec<String>) {
    if let Some(var) = env::var_os(key) {
        if let Ok(str) = var.into_string() {
            let mut str = str.to_owned();
            str.push_str(to_push);
            vec.push(str);
        }
    }
}

fn parse_args() -> Config {
    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} [options] [keymap.toml]", program);
        println!("{}", opts.usage(&brief));
    }

    let args: Vec<_> = env::args().collect();
    
    let mut default_configs = Vec::new();
    get_env_push("USERPROFILE", "\\keymap.toml", &mut default_configs);
    get_env_push("APPDATA", "\\keymap.toml", &mut default_configs);
    
    default_configs.push("keymap.toml".to_string());
    
    let c_msg = format!("specify the keymap config file to use (default in order: {:?})", default_configs);

    let mut opts = Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optflag("v", "version", "prints the version");
    opts.optopt("c", "config", &c_msg, "FILE");

    let matches = opts.parse(&args[1..]);
    if matches.is_err() {
        print_usage(&args[0], opts);
        exit(1);
    }
    let matches = matches.unwrap();
    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        exit(0);
    }

    if matches.opt_present("v") {
        println!("rusty-keys {}", VERSION);
        exit(0);
    }

    let config_file = matches.opt_str("c").unwrap_or_else(|| {
        let remaining_args = matches.free;
        if remaining_args.len() > 0 {
            remaining_args[0].clone()
        } else {
            for keymap in default_configs.drain(..) {
                if File::open(&keymap).is_ok() {
                    return keymap;
                }
            }
            println!("Error: no keymap.toml found...");
            print_usage(&args[0], opts);
            exit(1);
        }
    });

    Config::new(config_file)
}

use libc;

// Opaque Pointer Types
pub type Pointer = *mut libc::c_void;
pub type CGEventRef = Pointer;
pub type CFMachPortRef = Pointer;

// Integer Types
pub type CGEventField = u32;
pub type CGEventMask = u64;
pub type CGEventTapLocation = u32;
pub type CGEventTapOptions = u32;
pub type CGEventTapPlacement = u32;
pub type CGEventType = u32;
//pub type CGKeyCode = u16;

// Callback Type
pub type CGEventTapCallBack = extern "C"
fn(Pointer, CGEventType, CGEventRef, &mut MacOSKeyMaps) -> CGEventRef;

// Constants
/*
pub const kCGEventKeyDown: CGEventType = CGEventType::KeyDown;
pub const kCGEventKeyUp: CGEventType = CGEventType::KeyUp;
pub const kCGEventFlagsChanged: CGEventType = CGEventType::FlagsChanged;
pub const kCGSessionEventTap: CGEventTapLocation = 1;
pub const kCGHeadInsertEventTap: CGEventTapPlacement = 0;
pub const kCGKeyboardEventKeycode: CGEventField = 9;
pub const kCGEventTapDisabledByTimeout: CGEventType = CGEventType::TapDisabledByTimeout;
*/
pub const kCGEventKeyDown: CGEventType = 10;
pub const kCGEventKeyUp: CGEventType = 11;
pub const kCGEventFlagsChanged: CGEventType = 12;
pub const kCGSessionEventTap: CGEventTapLocation = 1;
pub const kCGHeadInsertEventTap: CGEventTapPlacement = 0;
pub const kCGKeyboardEventKeycode: CGEventField = 9;
pub const kCGEventTapDisabledByTimeout: CGEventType = 0xFFFFFFFE;

    // Link to ApplicationServices/ApplicationServices.h and Carbon/Carbon.h
    #[link(name = "ApplicationServices", kind = "framework")]
    #[link(name = "Carbon", kind = "framework")]
    extern {

        /// Pass through to the default loop modes
        pub static kCFRunLoopCommonModes: Pointer;

        /// Pass through to the default allocator
        pub static kCFAllocatorDefault: Pointer;

        /// Run the current threads loop in default mode
        pub fn CFRunLoopRun();

        /// Obtain the current threads loop
        pub fn CFRunLoopGetCurrent() -> Pointer;

        /// Get the code of the event back, e.g. the key code
        pub fn CGEventGetIntegerValueField(
            event: CGEventRef,
            field: CGEventField,
        ) -> CGKeyCode;

        /// Create an event tap
        ///
        /// # Arguments
        ///
        /// * `place` - The location of the new event tap. Pass one of
        ///          the constants listed in Event Tap Locations. Only
        ///          processes running as the root user may locate an
        ///          event tap at the point where HID events enter the
        ///          window server; for other users, this function
        ///          returns NULL.
        ///
        /// * `options` - The placement of the new event tap in the
        ///          list of active event taps. Pass one of the
        ///          constants listed in Event Tap Placement.
        ///
        /// * `eventsOfInterest` - A constant that specifies whether
        ///          the new event tap is a passive listener or an
        ///          active filter.
        ///
        /// * `callback` - A bit mask that specifies the set of events
        ///          to be observed. For a list of possible events,
        ///          see Event Types. For information on how to
        ///          specify the mask, see CGEventMask. If the event
        ///          tap is not permitted to monitor one or more of
        ///          the events specified in the eventsOfInterest
        ///          parameter, then the appropriate bits in the mask
        ///          are cleared. If that action results in an empty
        ///          mask, this function returns NULL.  callback
        ///
        /// * `refcon` - An event tap callback function that you
        ///          provide. Your callback function is invoked from
        ///          the run loop to which the event tap is added as a
        ///          source. The thread safety of the callback is
        ///          defined by the run loop’s environment. To learn
        ///          more about event tap callbacks, see
        ///          CGEventTapCallBack.  refcon
        ///
        /// * `channel` - A pointer to user-defined data. This pointer
        ///          is passed into the callback function specified in
        ///          the callback parameter.  Here we use it as a mpsc
        ///          channel.
        pub fn CGEventTapCreate(
            tap: CGEventTapLocation,
            place: CGEventTapPlacement,
            options: CGEventTapOptions,
            eventsOfInterest: CGEventMask,
            callback: CGEventTapCallBack,
            channel: &MacOSKeyMaps,
        ) -> CFMachPortRef;

        /// Creates a CFRunLoopSource object for a CFMachPort
        /// object.
        ///
        /// The run loop source is not automatically added to
        /// a run loop. To add the source to a run loop, use
        /// CFRunLoopAddSource
        pub fn CFMachPortCreateRunLoopSource(
            allocator: Pointer,
            port: CFMachPortRef,
            order: libc::c_int,
        ) -> Pointer;

        /// Adds a CFRunLoopSource object to a run loop mode.
        pub fn CFRunLoopAddSource(
            run_loop: Pointer,
            run_loop_source: Pointer,
            mode: Pointer,
        );

        pub fn CGEventTapEnable(port: CFMachPortRef, enable: bool);
    }

///  This callback will be registered to be invoked from the run loop
///  to which the event tap is added as a source.
#[no_mangle]
#[allow(unused_variables)]
pub extern fn callback(proxy: Pointer, event_type: CGEventType, event: CGEventRef, key_maps: &mut MacOSKeyMaps)
                       -> CGEventRef {


    match event_type {
        kCGEventKeyDown => println!("key down"),
        kCGEventKeyUp => println!("key up"),
        kCGEventFlagsChanged => println!("flags changed"),
        kCGEventTapDisabledByTimeout => {
            println!("Quartz event tap disabled because of timeout; attempting to reregister.");
            //register_listener(channel);
            //return event;
        },
        _ => {
            println!("Received unknown EventType: {}", event_type as u32);
            //return event;
        },
    };

    unsafe {
        let mut input_event = InputEvent {
            event_type,
            event,
        };
        println!("got keyCode: {}", input_event.code());
        //Some(input_event.event)
        //input_event.event
        key_maps.send_event(&mut input_event, &DEVICE).expect("macos shouldn't error...")
            .unwrap_or_else(|| std::ptr::null_mut()) // None means return NULL
        //let keyCode = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
        //println!("got keyCode: {}", keyCode);
        /*
        let event = KeyEvent {
            etype: match etype as u32 {
                kCGEventKeyDown => EventType::KeyDown,
                kCGEventKeyUp => EventType::KeyUp,
                kCGEventFlagsChanged => EventType::FlagsChanged,
                kCGEventTapDisabledByTimeout => {
                    warn!("Quartz event tap disabled because of timeout; attempting to reregister.");
                    register_listener(channel);
                    return event;
                },
                _ => {
                    error!("Received unknown EventType: {:}", etype);
                    return event;
                },
            },
            code: keyCode,
        };
        println!("Received event: {:?}", event);
        let _ = channel.send(event);
        */
    }
    //event
}

/// Redefine macro for bitshifting from header as function here
pub fn CGEventMaskBit(eventType: u32) -> CGEventMask {
    1 << (eventType)
}

