use core_graphics::display::CGDisplay;

pub fn display_size() -> (u64, u64) {
    let main = CGDisplay::main();
    (main.pixels_wide(), main.pixels_high())
}
