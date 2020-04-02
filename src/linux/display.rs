use crate::linux::common::Display;
use crate::rdev::DisplayError;
use std::convert::TryInto;

pub fn display_size() -> Result<(u64, u64), DisplayError> {
    let dpy = Display::new().map_err(|| DisplayError::NoDisplay)?;
    let screen = dpy.get_screen()?;
    Ok(
        screen.width.try_into().unwrap(),
        screen.height.try_into().unwrap(),
    )
}
