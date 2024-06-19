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
use log::{error, info};
use smart_leds::{brightness, colors::*, SmartLedsWrite, RGB8};
use std::{iter, thread, time::Duration};
use ws2812_spi::Ws2812;
use embedded_io_adapters::std::FromStd;
use ufmt::{uwrite, uwriteln};


use embedded_cli::cli::{CliBuilder, CliHandle};
use embedded_cli::Command;
use embedded_io;
use std::convert::Infallible;
use std::io;
use std::io::{BufRead, Read, stdin};
use std::ptr::null_mut;
use embedded_io::Write;
use esp_idf_svc::sys as _;
use esp_idf_svc::sys::{esp, esp_vfs_dev_uart_use_driver, uart_driver_install};


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

pub struct Writer (FromStd<io::Stdout>);

impl embedded_io::ErrorType for Writer {
    type Error = Infallible;
}

impl embedded_io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).unwrap();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().unwrap();
        Ok(())
    }
}


fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    //
    // unsafe {
    //     esp!(uart_driver_install(0, 512, 512, 10, null_mut(), 0)).unwrap();
    //     esp_vfs_dev_uart_use_driver(0);
    // }
    
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
    embedded_writer.write_all(b"Writing using embedded_io\n").unwrap();
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


    loop {
        thread::sleep(Duration::from_millis(50));
        let mut buf = [0u8; 1];
        if let Err(e) = stdin().read_exact(&mut buf) {
            info!("Error reading from stdin: {}", e);
            continue;
        } else {
            error!("Read byte: {}", buf[0]);
        }
        let byte = buf[0];

        // Process incoming byte
        // Command type is specified for autocompletion and help
        // Processor accepts closure where we can process parsed command
        // we can use different command and processor with each call
        let _ = cli.process_byte::<Base, _>(
            byte,
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
