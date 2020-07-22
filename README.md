# AT42QT1070

This is an I2C implementation for the [AT42QT1070](https://www.microchip.com/wwwproducts/en/AT42QT1070) Touch Sensor IC.

The used I2C struct is required to implement [embedded_hal::blocking::i2c::Write](https://docs.rs/embedded-hal/0.2.4/embedded_hal/blocking/i2c/trait.Write.html) and [embedded_hal::blocking::i2c::WriteRead](https://docs.rs/embedded-hal/0.2.4/embedded_hal/blocking/i2c/trait.WriteRead.html)


## [Example](https://github.com/Steav005/at42qt1070/blob/master/examples/stm32f401cc.rs)

```rust
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
```
