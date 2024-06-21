mod cli;
mod led;
mod vfs_embedded_reader;

use crate::cli::{configure_serial, console_task};
use crate::led::led_task;
use anyhow::Result;
use embedded_io_adapters::std::FromStd;
use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    peripherals::Peripherals,
    spi::{config::DriverConfig, Dma, SpiDriver},
    task::thread::ThreadSpawnConfiguration,
};
use std::{io, os::fd::AsRawFd, thread, time::Duration};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    configure_serial();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let driver = SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio10,
        Option::<AnyIOPin>::None,
        &DriverConfig::new().dma(Dma::Auto(512)),
    )?;

    // High-prio tasks
    ThreadSpawnConfiguration {
        priority: 7,
        ..Default::default()
    }
    .set()
    .unwrap();
    let mut reader = vfs_embedded_reader::VfsReader::new(io::stdin().as_raw_fd());
    let writer = FromStd::new(io::stdout());
    let _handle = thread::spawn(move || console_task(&mut reader, writer));

    // Low-prio tasks
    ThreadSpawnConfiguration {
        priority: 4,
        ..Default::default()
    }
    .set()
    .unwrap();
    let _handle = thread::spawn(move || led_task(driver));

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
