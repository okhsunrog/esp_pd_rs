use anyhow::Result;
use embedded_io_adapters::std::FromStd;
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
use log::error;
use smart_leds::{brightness, colors::*, SmartLedsWrite, RGB8};
use std::{iter, thread, time::Duration};
use ufmt::uwrite;
use ws2812_spi::Ws2812;

use embedded_cli::cli::CliBuilder;
use embedded_cli::Command;
use std::ffi::c_void;
use std::io;
use std::io::stdin;
use std::os::fd::AsRawFd;

#[derive(Command)]
enum Base<'a> {
    Hello { name: Option<&'a str> },
    Exit,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        use esp_idf_svc::sys::{
            esp_vfs_usb_serial_jtag_use_driver, usb_serial_jtag_driver_config_t,
            usb_serial_jtag_driver_install,
        };
        let mut serial_config = usb_serial_jtag_driver_config_t {
            rx_buffer_size: 128,
            tx_buffer_size: 128,
        };
        usb_serial_jtag_driver_install(&mut serial_config);
        esp_vfs_usb_serial_jtag_use_driver();
    }
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let driver = SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio10,
        Option::<AnyIOPin>::None,
        &DriverConfig::new().dma(Dma::Auto(512)),
    )?;

    let _handle = thread::spawn(move || blink_task(driver));

    let embedded_writer: FromStd<io::Stdout> = FromStd::new(io::stdout());
    let mut cli = CliBuilder::default()
        .writer(embedded_writer)
        .build()
        .unwrap();

    cli.write(|writer| {
        uwrite!(writer, "Cli is running.")?;
        Ok(())
    })
    .unwrap();

    let reader_fd = stdin().as_raw_fd();
    let mut buf = [0u8];
    loop {
        thread::sleep(Duration::from_millis(50));
        use esp_idf_svc::sys::read;
        let ret = unsafe { read(reader_fd, buf.as_mut_ptr() as *mut c_void, 1) };
        match ret {
            -1 => {
                error!("Error reading from stdin");
                continue;
            }
            0 => continue,
            _ => {}
        }

        let _ = cli.process_byte::<Base, _>(
            buf[0],
            &mut Base::processor(|cli, command| {
                match command {
                    Base::Hello { name } => {
                        uwrite!(cli.writer(), "Hello, {}", name.unwrap_or("World"))?;
                    }
                    Base::Exit => {
                        cli.writer().write_str("Cli can't shutdown now")?;
                    }
                }
                Ok(())
            }),
        );
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
