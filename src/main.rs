mod cli;
mod display;
mod led;
mod vfs_embedded_reader;

use anyhow::Result;
use embedded_io_adapters::std::FromStd;
use esp_idf_svc::hal::{
    prelude::*,
    i2c::{I2cConfig, I2cDriver},
    gpio::AnyIOPin,
    peripherals::Peripherals,
    spi::{config::DriverConfig, Dma, SpiDriver},
    task::thread::ThreadSpawnConfiguration,
};
use std::{io, os::fd::AsRawFd, thread};
use crate::{
    cli::{configure_serial, console_task},
    display::display_task,
    led::led_task,
};


fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    configure_serial();
    esp_idf_svc::log::EspLogger::initialize_default();
    // setting up peripherals
    let peripherals = Peripherals::take()?;
    // spi for ws2812b
    let driver = SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio10,
        Option::<AnyIOPin>::None,
        &DriverConfig::new().dma(Dma::Auto(512)),
    )?;
    // i2c for fusb302 and ssd1306
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    // High-prio tasks
    ThreadSpawnConfiguration {
        priority: 7,
        ..Default::default()
    }.set()?;
    let mut reader = vfs_embedded_reader::VfsReader::new(io::stdin().as_raw_fd());
    let writer = FromStd::new(io::stdout());
    thread::spawn(move || console_task(&mut reader, writer));

    // Low-prio tasks
    ThreadSpawnConfiguration {
        priority: 4,
        ..Default::default()
    }.set()?;
    thread::spawn(move || led_task(driver));
    thread::spawn(move || display_task(i2c));

    Ok(())
}
