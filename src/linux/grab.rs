use crate::linux::keycodes::key_from_code;
use crate::linux::simulate::simulate;
use crate::rdev::{Button, Event, EventType, GrabCallback, GrabError, Key};
use std::mem::transmute;
use std::time::SystemTime;
use xcb::ffi::base::XCB_NONE;
use xcb::{
    create_window, destroy_window, grab_keyboard, grab_pointer, ungrab_keyboard, ungrab_pointer,
    ButtonPressEvent, ButtonReleaseEvent, Connection, KeyPressEvent, KeyReleaseEvent,
    MotionNotifyEvent, BUTTON_PRESS, BUTTON_RELEASE, COPY_FROM_PARENT, CURRENT_TIME, ENTER_NOTIFY,
    EVENT_MASK_POINTER_MOTION, EXPOSE, GRAB_MODE_ASYNC, GRAB_STATUS_SUCCESS, KEY_PRESS,
    KEY_RELEASE, LEAVE_NOTIFY, MOTION_NOTIFY, WINDOW_CLASS_INPUT_ONLY,
};

fn xcb_event_to_rdev_event(event: xcb::GenericEvent) -> Option<EventType> {
    match event.response_type() {
        KEY_PRESS => {
            let key_event: KeyPressEvent = unsafe { transmute(event) };
            let key = key_from_code(key_event.detail().into());
            Some(EventType::KeyPress(key))
        }
        KEY_RELEASE => {
            let key_event: KeyReleaseEvent = unsafe { transmute(event) };
            let key = key_from_code(key_event.detail().into());
            Some(EventType::KeyRelease(key))
        }
        BUTTON_PRESS => {
            let mouse_event: ButtonPressEvent = unsafe { transmute(event) };
            let button = match mouse_event.detail() {
                1 => Button::Left,
                2 => Button::Middle,
                3 => Button::Right,
                code => Button::Unknown(code),
            };
            Some(EventType::ButtonPress(button))
        }
        BUTTON_RELEASE => {
            let mouse_event: ButtonReleaseEvent = unsafe { transmute(event) };
            let button = match mouse_event.detail() {
                1 => Button::Left,
                2 => Button::Middle,
                3 => Button::Right,
                code => Button::Unknown(code),
            };
            Some(EventType::ButtonRelease(button))
        }
        MOTION_NOTIFY => {
            let mouse_event: MotionNotifyEvent = unsafe { transmute(event) };
            Some(EventType::MouseMove {
                x: mouse_event.root_x().into(),
                y: mouse_event.root_y().into(),
            })
        }
        EXPOSE | ENTER_NOTIFY | LEAVE_NOTIFY => None,
        _ => None, // this should never happen
    }
}

pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    let (conn, screen_num) = Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let root = screen.root();
    let window = conn.generate_id();

    create_window(
        &conn,
        COPY_FROM_PARENT as u8,
        window,
        root,
        10,
        10,
        0,
        0,
        0,
        WINDOW_CLASS_INPUT_ONLY as u16,
        screen.root_visual(),
        &[],
    );
    xcb::map_window(&conn, window);
    conn.flush();

    println!("before grab");
    let kbd_cookie = grab_keyboard(
        &conn,
        false,
        root,
        CURRENT_TIME,
        GRAB_MODE_ASYNC as u8,
        GRAB_MODE_ASYNC as u8,
    );
    let ptr_cookie = grab_pointer(
        &conn,
        true,
        root,
        EVENT_MASK_POINTER_MOTION as u16,
        GRAB_MODE_ASYNC as u8,
        GRAB_MODE_ASYNC as u8,
        XCB_NONE,
        XCB_NONE,
        CURRENT_TIME,
    );
    let grab_err = |_| GrabError::LinuxNotSupported;
    if kbd_cookie.get_reply().map_err(grab_err)?.status() != GRAB_STATUS_SUCCESS as u8 {
        return Err(GrabError::LinuxNotSupported);
    }
    if ptr_cookie.get_reply().map_err(grab_err)?.status() != GRAB_STATUS_SUCCESS as u8 {
        return Err(GrabError::LinuxNotSupported);
    }
    while let Some(event) = conn.wait_for_event() {
        let event_type = xcb_event_to_rdev_event(event);
        println!("grabbed_event: {:?}", event_type);
        if event_type == Some(EventType::KeyPress(Key::Escape)) {
            // TODO: remove
            // used in testing to prevent
            break;
        }
        let event = event_type.map(|event_type| Event {
            event_type,
            time: SystemTime::now(),
            name: None,
        });
        let maybe_event = event.map(|ev| callback(ev)).flatten();
        match maybe_event {
            Some(event) => {
                //simulate(&event.event_type)?;
            }
            None => {}
        };
    }
    // This code should never run, but it is the correct cleanup procedure
    ungrab_pointer(&conn, CURRENT_TIME)
        .request_check()
        .map_err(|_| GrabError::LinuxNotSupported)?;
    ungrab_keyboard(&conn, CURRENT_TIME)
        .request_check()
        .map_err(|_| GrabError::LinuxNotSupported)?;
    destroy_window(&conn, window)
        .request_check()
        .map_err(|_| GrabError::LinuxNotSupported)?;
    Ok(())
}
