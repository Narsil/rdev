use objc2_core_graphics::{CGDisplayPixelsHigh, CGDisplayPixelsWide, CGMainDisplayID};

use crate::rdev::DisplayError;

pub fn display_size() -> Result<(u64, u64), DisplayError> {
    unsafe {
        let main = CGMainDisplayID();
        Ok((
            CGDisplayPixelsWide(main)
                .try_into()
                .map_err(|_| DisplayError::ConversionError)?,
            CGDisplayPixelsHigh(main)
                .try_into()
                .map_err(|_| DisplayError::ConversionError)?,
        ))
    }
}
