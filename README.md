Simple library to listen and send events to keyboard and mouse.

## Simple Usage

Listening to global events

```rust
use rdev::{listen, Event};

fn main() {
    listen(callback);
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
}
```

Sending some events

```rust
use rdev::{simulate, EventType, SimulateError};
use std::{thread, time};

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn main() {
    send(&EventType::KeyPress { code: 39 });
    send(&EventType::KeyRelease { code: 39 });

    send(&EventType::MouseMove { x: 0.0, y: 0.0 });
    send(&EventType::MouseMove { x: 400.0, y: 400.0 });
    send(&EventType::ButtonPress { code: 1 });
    send(&EventType::ButtonRelease { code: 1 });
    send(&EventType::Wheel {
        delta_x: 0,
        delta_y: 1,
    });
}
```

### Events enum

In order to manage different OS, the current EventType choices is a mix&match
to account for all possible events.

```rust
pub enum EventType {
    KeyPress { code: u8 },
    KeyRelease { code: u8 },
    // Note: On MacOS, only LeftButton and RightButton Press and Release are defined
    // They are currently mapped to ButtonPress{code: 1} and ButtonPress{code: 3} to
    // be closer to X11 behaviour
    ButtonPress { code: u8 },
    ButtonRelease { code: u8 },
    // Values in pixels
    MouseMove { x: i32, y: i32 },
    // Note: On Linux, there is no actual delta the actual values are ignored for delta_x
    // and we only look at the sign of delta_y to simulate wheelup or wheeldown.
    Wheel { delta_x: i64, delta_y: i64 },
}
```

### OS Specificities

For now the code only works for Linux (X11) and MacOS. On MacOS, the listen
loop needs to be the primary app (no fork before) and need to have accessibility
settings enabled. The `listen_and_simulate` test does have both a listen and a simulate part.
We use tokio to manage the listen process.
