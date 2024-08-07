use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::hal::spi::{config::Config, SpiBusDriver, SpiDriver};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use std::time::Duration;
use std::thread;
use smart_leds::hsv::{Hsv, hsv2rgb};
use ws2812_spi::Ws2812;

pub fn led_task(driver: SpiDriver) -> anyhow::Result<()> {
    let bus = SpiBusDriver::new(driver, &Config::new().baudrate(3_200.kHz().into()))?;
    let mut ws = Ws2812::new(bus);
    let mut data = [RGB8::default(); 1];
    #[allow(clippy::infinite_iter)]
    (0..=255).cycle().for_each(|hue| {
        thread::sleep(Duration::from_millis(10));
        data[0] = hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 120,
        });
        let pixel = brightness(data.iter().cloned(), 30);
        ws.write(pixel).unwrap();
    });
    Ok(())
}
