use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::hal::spi::{config::Config, SpiBusDriver, SpiDriver};
use heapless::Vec as HVec;
use smart_leds::colors::{BLACK, ORANGE};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use std::time::Duration;
use std::{iter, thread};
use ws2812_spi::Ws2812;

pub fn blink_task(driver: SpiDriver) -> anyhow::Result<()> {
    let bus = SpiBusDriver::new(driver, &Config::new().baudrate(3_200.kHz().into()))?;
    let mut ws = Ws2812::new(bus);
    let colors: HVec<RGB8, 2> = brightness([ORANGE, BLACK].into_iter(), 50).collect();
    for color in colors.into_iter().cycle() {
        ws.write(iter::once(color))?;
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
