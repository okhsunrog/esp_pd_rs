use std::time::Duration;
use embedded_cli::cli::CliBuilder;
use embedded_cli::Command;
use embedded_io::{Error, Read, Write};
use esp_idf_svc::sys;
use ufmt::uwrite;

#[derive(Command)]
enum Base<'a> {
    Hello { name: Option<&'a str> },
    Exit,
}

pub fn console_task<T, TE, U>(reader: &mut U, writer: T)
where
    T: Write<Error = TE>,
    TE: Error,
    U: Read,
{
    std::thread::sleep(Duration::from_millis(50));
    let mut cli = CliBuilder::default().writer(writer).build().unwrap();
    let mut buf = [0u8];
    loop {
        match reader.read(&mut buf) {
            Ok(1) => (),
            _ => continue,
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

pub fn configure_serial() {
    unsafe {
        let mut serial_config = sys::usb_serial_jtag_driver_config_t {
            rx_buffer_size: 128,
            tx_buffer_size: 128,
        };
        sys::usb_serial_jtag_driver_install(&mut serial_config);
        sys::esp_vfs_usb_serial_jtag_use_driver();
    }
}
