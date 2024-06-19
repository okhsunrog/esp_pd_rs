use anyhow::Result;
use esp_idf_svc::hal::{
    //all hal imports go here
    gpio::AnyIOPin,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        config::{Config, DriverConfig},
        Dma, SpiBusDriver, SpiDriver,
    },
};
use heapless::Vec as HVec;
use log::info;
use smart_leds::{brightness, colors::*, SmartLedsWrite, RGB8};
use std::{iter, thread, time::Duration};
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

    info!("Spawning new thread for the WS2812 LED");
    let _handle = thread::spawn(move || blink_task(driver));

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn blink_task(driver: SpiDriver) -> Result<()> {
    let bus = SpiBusDriver::new(driver, &Config::new().baudrate(3_200.kHz().into()))?;
    let mut ws = Ws2812::new(bus);
    let colors: HVec<RGB8, 2> = brightness([ORANGE, BLACK].into_iter(), 30).collect();
    for color in colors.into_iter().cycle() {
        ws.write(iter::once(color))?;
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
