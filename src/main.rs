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
use log::{error, info};
use smart_leds::{brightness, colors::*, SmartLedsWrite, RGB8};
use std::{iter, thread, time::Duration};
use ufmt::uwrite;
use ws2812_spi::Ws2812;

use embedded_cli::cli::CliBuilder;
use embedded_cli::Command;
use embedded_io;
use embedded_io::Write;
use std::ffi::c_void;
use std::io;
use std::io::stdin;
use std::os::fd::AsRawFd;

#[derive(Command)]
enum Base<'a> {
    /// Say hello to World or someone else
    Hello {
        /// To whom to say hello (World by default)
        name: Option<&'a str>,
    },

    /// Stop CLI and exit
    Exit,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        use esp_idf_svc::sys::{
            esp_vfs_usb_serial_jtag_use_driver,
            usb_serial_jtag_driver_config_t, usb_serial_jtag_driver_install,
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

    info!("Spawning new thread for the WS2812 LED");
    let _handle = thread::spawn(move || blink_task(driver));

    let mut embedded_writer: FromStd<io::Stdout> = FromStd::new(io::stdout());
    embedded_writer
        .write_all(b"Writing using embedded_io\n")
        .unwrap();
    let _ = embedded_writer.flush();
    println!("Writing using std");

    let mut cli = CliBuilder::default()
        .writer(embedded_writer)
        .build()
        .unwrap();

    cli.write(|writer| {
        uwrite!(
            writer,
            "Cli is running. Press 'Esc' to exit
Type \"help\" for a list of commands.
Use backspace and tab to remove chars and autocomplete.
Use up and down for history navigation.
Use left and right to move inside input."
        )?;
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
            },
            0 => continue,
            _ => {}
        }

        let _ = cli.process_byte::<Base, _>(
            buf[0],
            &mut Base::processor(|cli, command| {
                match command {
                    Base::Hello { name } => {
                        // last write in command callback may or may not
                        // end with newline. so both uwrite!() and uwriteln!()
                        // will give identical results
                        uwrite!(cli.writer(), "Hello, {}", name.unwrap_or("World"))?;
                    }
                    Base::Exit => {
                        // We can write via normal function if formatting not needed
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
