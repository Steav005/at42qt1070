#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

pub use rtic::{app, cyccnt::U32Ext};

use at42qt1070::*;

use stm32f4xx_hal::i2c::*;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::gpio::gpiob::{PB8, PB9};
use stm32f4xx_hal::gpio::gpioa::{PA1, PA2, PA3, PA4};
use stm32f4xx_hal::gpio::{AlternateOD, AF4, Output, PushPull, ExtiPin, Edge, Input, PullUp};
use stm32f4xx_hal::stm32::I2C1;
use stm32f4xx_hal::interrupt::EXTI4;

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        sensor: At42qt1070<I2c<I2C1, (PB8<AlternateOD<AF4>>, PB9<AlternateOD<AF4>>)>>,
        led_red: PA1<Output<PushPull>>,
        led_green: PA2<Output<PushPull>>,
        led_yellow: PA3<Output<PushPull>>,
        change_interrupt: PA4<Input<PullUp>>,
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

        //Initialize Leds
        let gpioa = c.device.GPIOA.split();
        let led_red = gpioa.pa1.into_push_pull_output();
        let led_green = gpioa.pa2.into_push_pull_output();
        let led_yellow = gpioa.pa3.into_push_pull_output();

        //Initialize Interrupt Input
        let mut syscfg = c.device.SYSCFG;
        let mut exti = c.device.EXTI;
        //Connected Change Line of IC (Pin 5) to PA4
        let mut change_interrupt = gpioa.pa4.into_pull_up_input();
        change_interrupt.make_interrupt_source(&mut syscfg);
        change_interrupt.trigger_on_edge(&mut exti, Edge::FALLING);
        change_interrupt.enable_interrupt(&mut exti);

        rtic::pend(EXTI4);
        loop{
            //Wait for Sensor being ready
            //Note: The CHANGE line is pulled low 100 ms after power-up or reset. //Chapter 2.7
            if change_interrupt.is_low().unwrap(){
                break;
            }
        }

        //Initialize Touch IC
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb9.into_alternate_af4().set_open_drain();

        let i2c = I2c::i2c1(c.device.I2C1, (scl, sda), 400.khz(), clocks);
        let mut sensor = At42qt1070::new(i2c);
        //Initial Sync
        sensor.sync_all().unwrap();
        //Set AKS to 0 for all Keys, so they are not Grouped
        for i in 0..7{
            sensor.set_aks(0, Key::from(i)).unwrap();
        }

        init::LateResources { sensor, led_red, led_green, led_yellow, change_interrupt }
    }

    #[task(binds = EXTI4, resources = [sensor, led_red, led_green, led_yellow, change_interrupt])]
    fn interrupt(c: interrupt::Context) {
        c.resources.change_interrupt.clear_interrupt_pending_bit();

        //Sync all (or ar least one keys status bytes for clearing the change line of the IC
        //Chapter 2.7
        c.resources.sensor.sync_all().unwrap();

        //Just read the cached status, because we just synced
        let status = c.resources.sensor.read_cached_full_key_status();

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
