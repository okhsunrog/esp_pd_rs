use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use esp_idf_svc::hal::i2c::I2cDriver;

pub fn display_task(i2c: I2cDriver) {
    use ssd1315::{config, interface, Ssd1315};
    let interface = interface::I2cDisplayInterface::new_interface(i2c);
    let config = config::Ssd1315DisplayConfig::preset_config();

    let mut display = Ssd1315::new(interface);
    display.set_custom_config(config);

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);
    let im = Image::new(&raw, Point::new(32, 0));
    im.draw(&mut display).unwrap();

    display.init_screen();
    display.flush_screen();
}