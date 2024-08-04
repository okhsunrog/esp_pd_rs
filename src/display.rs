use embedded_graphics::{
    geometry::Point,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    text::Text,
    Drawable,
};
use esp_idf_svc::hal::i2c::I2cDriver;
use ssd1315::{config, interface, Ssd1315};

pub fn display_task(i2c: I2cDriver) {
    let interface = interface::I2cDisplayInterface::new_interface(i2c);
    let config = config::Ssd1315DisplayConfig::preset_config();

    let mut display = Ssd1315::new(interface);
    display.set_custom_config(config);

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello World", Point::new(10, 20), text_style)
        .draw(&mut display)
        .unwrap();

    display.init_screen();
    display.flush_screen();
}
