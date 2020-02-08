use crate::rdev::{Event, EventType};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use std::time::SystemTime;

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        tap_type: u32,
        tap_option: u32,
        tap_mask: u32,
        callback: fn(),
        no_idea: Option<u32>,
    );
    fn CFMachPortRunLoopSource();
    fn CFRunLoopAddSource();
    fn CFRunLoopGetCurrent();

}
struct CGEvent;
pub const kCGHIDEventTap: u32 = 0;
pub const kCGHeadInsertEventTap: u32 = 0;
pub const kCGEventTapOptionDefault: u32 = 0x00000000;
pub const kCGEventMaskForAllEvents: u32 = 0;

struct EventIterator;

impl EventIterator {
    fn wait_next_event(&mut self) -> Event {
        let name = None;
        let code = 1;
        let event_type = EventType::KeyPress { code };
        Event::new(event_type, SystemTime::now(), name).unwrap()
    }
}

impl Iterator for EventIterator {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        Some(self.wait_next_event())
    }
}

#[link(name = "Cocoa", kind = "framework")]
fn listen() -> EventIterator {
    fn callback(cgevent: CGEvent) {
        println!("{:?}", cgevent);
    }
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            kCGHIDEventTap,
            kCGHeadInsertEventTap,
            kCGEventTapOptionDefault,
            kCGEventMaskForAllEvents,
            callback,
            None,
        );
    }
    EventIterator {}

    // if not tap:
    //     sys.exit(1)

    // loop = CFMachPortCreateRunLoopSource(None, tap, 0)
    // CFRunLoopAddSource(CFRunLoopGetCurrent(), loop, kCFRunLoopCommonModes)

    // CGEventTapEnable(tap, True)
    // self.lastFlags = 0
    // self.kbstate = KeyboardState()

    // CFRunLoopRun()
}
