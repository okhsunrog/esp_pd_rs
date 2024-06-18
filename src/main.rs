use anyhow::Result;
use config::{Config, DriverConfig};
use esp_idf_svc::hal::gpio::AnyIOPin;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::*;
use esp_idf_svc::hal::spi::{SpiBusDriver, SpiDriver};
use log::info;
use smart_leds::{SmartLedsWrite, RGB8};
use std::thread;
use std::time::Duration;
use ws2812_spi::Ws2812;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let sclk = peripherals.pins.gpio6;
    let sdo = peripherals.pins.gpio10;

    let driver = SpiDriver::new(
        spi,
        sclk,
        sdo,
        Option::<AnyIOPin>::None,
        &DriverConfig::new(),
    )
    .unwrap();

    let config = Config::new().baudrate(3_200_u32.kHz().into());

    let bus = SpiBusDriver::new(driver, &config).unwrap();

    let mut data: [RGB8; 1] = [RGB8::default(); 1];
    let empty: [RGB8; 1] = [RGB8::default(); 1];
    let mut ws = Ws2812::new(bus);

    loop {
        data[0] = RGB8 {
            r: 0,
            g: 0x10,
            b: 0,
        };
        ws.write(data.iter().cloned()).unwrap();
        thread::sleep(Duration::from_secs(2));
        info!("on");

        ws.write(empty.iter().cloned()).unwrap();
        thread::sleep(Duration::from_secs(2));
        info!("off");
    }
}
