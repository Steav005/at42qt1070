#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

pub use rtic::app;

use stm32f4xx_hal::i2c::*;

use stm32f4xx_hal::prelude::*;

use at42qt1070::*;
use stm32f4xx_hal::gpio::gpiob::{PB8, PB9};
use stm32f4xx_hal::gpio::gpioa::{PA1, PA2, PA3};
use stm32f4xx_hal::gpio::{AlternateOD, AF4, Output, PushPull};
use stm32f4xx_hal::stm32::{I2C1, TIM3};
use stm32f4xx_hal::timer::{Timer, Event};

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        sensor: At42qt1070<I2c<I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>,
        led_red: PA1<Output<PushPull>>,
        led_green: PA2<Output<PushPull>>,
        led_yellow: PA3<Output<PushPull>>,
        timer: Timer<TIM3>
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

        //Initialize Touch IC
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().set_open_drain();

        let i2c = I2c::i2c1(c.device.I2C1, (scl, sda), 400.khz(), clocks);
        let mut sensor = At42qt1070::new(i2c);
        //Initial Sync
        sensor.sync_all().unwrap();
        //Set AKS to 0
        for i in 0..7{
            sensor.set_aks(0, Key::from(i)).unwrap();
        }


        //Initialize Leds
        let gpioa = c.device.GPIOA.split();
        let led_red = gpioa.pa1.into_push_pull_output();
        let led_green = gpioa.pa2.into_push_pull_output();
        let led_yellow = gpioa.pa3.into_push_pull_output();

        //Initialize Timer
        let mut timer = Timer::tim3(c.device.TIM3, 1.khz(), clocks);
        timer.listen(Event::TimeOut);

        init::LateResources { sensor, led_red, led_green, led_yellow, timer }
    }

    #[task(binds = TIM3, resources = [sensor, led_red, led_green, led_yellow, timer])]
    fn periodic(c: periodic::Context) {
        c.resources.timer.clear_interrupt(Event::TimeOut);

        let status = c.resources.sensor.read_full_key_status().unwrap();

        if status[1] {
            c.resources.led_red.set_high().unwrap();
        } else {
            c.resources.led_red.set_low().unwrap();
        }

        if status[2] {
            c.resources.led_yellow.set_high().unwrap();
        } else {
            c.resources.led_yellow.set_low().unwrap();
        }

        if status[3] {
            c.resources.led_green.set_high().unwrap();
        } else {
            c.resources.led_green.set_low().unwrap();
        }
    }
};
