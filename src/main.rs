use anyhow::Result;
use config::{Config, DriverConfig};
use esp_idf_svc::hal::gpio::AnyIOPin;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::*;
use esp_idf_svc::hal::spi::{SpiBusDriver, SpiDriver};
use log::info;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{SmartLedsWrite, RGB8};
use std::thread;
use std::time::Duration;
use ws2812_spi::Ws2812;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take()?;

    let driver = SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio10,
        Option::<AnyIOPin>::None,
        &DriverConfig::new().dma(Dma::Auto(512)),
    )?;

    let config = Config::new().baudrate(3_200.kHz().into());
    let bus = SpiBusDriver::new(driver, &config)?;

    let mut data: [RGB8; 1] = [RGB8::default(); 1];
    let mut ws = Ws2812::new(bus);
    info!("Running rainbow test...");

    #[allow(clippy::infinite_iter)]
    (0..=255).cycle().for_each(|hue| {
        thread::sleep(Duration::from_millis(10));
        data[0] = hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 120,
        });
        ws.write(data.iter().cloned()).unwrap();
    });
    Ok(())
}
