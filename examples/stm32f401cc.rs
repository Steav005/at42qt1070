#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

pub use rtic::app;

use stm32f4xx_hal::i2c::*;

use stm32f4xx_hal::prelude::*;

use at42qt1070::*;
use stm32f4xx_hal::stm32::I2C1;
use stm32f4xx_hal::gpio::{AF4, AlternateOD};
use stm32f4xx_hal::gpio::gpiob::{PB8, PB9};

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        sensor: At42qt1070<I2c<I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>,
    }

    #[init]
    fn init(c: init::Context) -> init::LateResources {
        let rcc = c.device.RCC.constrain();
        let gpiob = c.device.GPIOB.split();

        let clocks = rcc
            .cfgr
            .use_hse(25.mhz())
            .sysclk(84.mhz())
            .require_pll48clk()
            .freeze();

        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().set_open_drain();

        let i2c = I2c::i2c1(c.device.I2C1, (scl, sda), 400.khz(), clocks);
        let mut sensor = At42qt1070::new(i2c);
        sensor.sync_all().unwrap();

        init::LateResources {
            sensor,
        }
    }
};