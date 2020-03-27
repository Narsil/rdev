![](https://github.com/Narsil/rdev/workflows/build/badge.svg)
[![Crate](https://img.shields.io/crates/v/rdev.svg)](https://crates.io/crates/rdev)
[![API](https://docs.rs/rdev/badge.svg)](https://docs.rs/rdev)

Simple library to listen and send events to keyboard and mouse.

You can also check out [Enigo](https://github.com/Enigo-rs/Enigo) which is another
crate which helped me write this one.

This crate is so far a pet project for me to understand the rust ecosystem.

## Simple Usage

Listening to global events

```rust
use rdev::{listen, Event};

fn main() {
    // This will block.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
    match event.name {
        Some(string) => println!("User wrote {:?}", string),
        None => (),
    }
}
```

Sending some events

```rust
use rdev::{simulate, Button, EventType, Key, SimulateError};
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
    send(&EventType::KeyPress(Key::KeyS));
    send(&EventType::KeyRelease(Key::KeyS));

    send(&EventType::MouseMove { x: 0.0, y: 0.0 });
    send(&EventType::MouseMove { x: 400.0, y: 400.0 });
    send(&EventType::ButtonPress(Button::Left));
    send(&EventType::ButtonRelease(Button::Right));
    send(&EventType::Wheel {
        delta_x: 0,
        delta_y: 1,
    });
}
```

Getting the main screen size

```rust
use rdev::{display_size};

fn main() {
    let (w, h) = display_size();
    assert!(w > 0);
    assert!(h > 0);
}
```

Serialization

Serialization and deserialization is optional behind the feature "serialize".

### Event struct

In order to detect what a user types, we need to plug to the OS level management
of keyboard state (modifiers like shift, ctrl, but also dead keys if they exist).

In order to see what is the outcome of an event, you need to read the Event::name option.

```rust
/// When events arrive from the system we can add some information
/// time is when the event was received, name *will* be at some point changed
/// to be mapped to the function of the key (Alt, s, Return and so on).
#[derive(Debug)]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}
```

Be careful, Event::name, might be None, but also String::from(""), and might contain
not displayable unicode characters. We send exactly what the OS sends us so do some sanity checking
before using it.
Caveat: Dead keys don't function yet on Linux

### Events enum

In order to manage different OS, the current EventType choices is a mix&match
to account for all possible events.
There is a safe mechanism to detect events no matter what, which are the
Unknown() variant of the enum which will contain some OS specific value.
Also not that not all keys are mapped to an OS code, so simulate might fail if you
try to send an unmapped key. Sending Unknown() variants will always work (the OS might
still reject it).

```rust
/// In order to manage different OS, the current EventType choices is a mix&match
/// to account for all possible events.
#[derive(Debug)]
pub enum EventType {
    /// The keys correspond to a standard qwerty layout, they don't correspond
    /// To the actual letter a user would use, that requires some layout logic to be added.
    KeyPress(Key),
    KeyRelease(Key),
    /// Some mouse will have more than 3 buttons, these are not defined, and different OS will
    /// give different Unknown code.
    ButtonPress(Button),
    ButtonRelease(Button),
    /// Values in pixels
    MouseMove {
        x: f64,
        y: f64,
    },
    /// Note: On Linux, there is no actual delta the actual values are ignored for delta_x
    /// and we only look at the sign of delta_y to simulate wheelup or wheeldown.
    Wheel {
        delta_x: i64,
        delta_y: i64,
    },
}
```

### OS Specificities

For now the code only works for Linux (X11), MacOS and Windows. On MacOS, the listen
loop needs to be the primary app (no fork before) and need to have accessibility
settings enabled. The `listen_and_simulate` test does have both a listen and a simulate part.
We use tokio to manage the listen process (Terminal was added in accessibility settings).
