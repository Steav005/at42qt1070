#![no_std]
#![allow(dead_code)]

use embedded_hal::blocking::i2c;

mod register_map;
use crate::register_map::Register::*;
use core::time::Duration;
pub use register_map::*;

// http://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-9596-AT42-QTouch-BSW-AT42QT1070_Datasheet.pdf
// Chapter 4.2
const AT42QT1070_I2C_ADDR: u8 = 0x1B;

pub struct At42qt1070<I2C> {
    i2c: I2C,
    register_map: RegisterMap,
}

impl<I2C, E> At42qt1070<I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> At42qt1070<I2C> {
        let register_map = RegisterMap::default();
        At42qt1070 { i2c, register_map }
    }

    pub fn release(self) -> I2C {
        self.i2c
    }

    pub fn device_reg(&self, reg: &Register) -> u8 {
        self.register_map.reg_as_byte(reg)
    }

    pub fn wait_calibrated_blocking(&mut self) -> Result<(), E> {
        loop {
            if !self.read_detection_status()?.2 {
                return Ok(())
            }
        }
    }

    pub fn set_negative_threshold(&mut self, threshold: u8, key: Key) -> Result<(), E> {
        self.write_reg_map_reg(&NthrKey(key), threshold)?;
        *self.register_map.get_nthr_key_register_mut(&key) = threshold;
        Ok(())
    }

    pub fn set_ave_aks(&mut self, ave: u8, aks: u8, key: Key) -> Result<(), E> {
        let value = AveAks{ave, aks}.as_byte();
        self.write_reg_map_reg(&AveAksKey(key), value)?;
        self.register_map.get_ave_aks_key_register_mut(&key).update(value);

        Ok(())
    }

    pub fn set_ave(&mut self, ave: u8, key: Key) -> Result<(), E> {
        let aks = self.read_cached_ave_aks(key).1;
        self.set_ave_aks(ave, aks, key)
    }

    pub fn set_aks(&mut self, aks: u8, key: Key) -> Result<(), E> {
        let ave = self.read_cached_ave_aks(key).0;
        self.set_ave_aks(ave, aks, key)
    }

    pub fn set_detection_integrator(&mut self, di: u8, key: Key) -> Result<(), E> {
        self.write_reg_map_reg(&DIKey(key), di)?;
        *self.register_map.get_di_key_register_mut(&key) = di;

        Ok(())
    }

    pub fn set_fo_mc_guard(&mut self, fast_out: bool, max_cal: bool, guard_channel: Option<Key>) -> Result<(), E> {
        let guard_channel = match guard_channel {
            Some(key) => key as u8,
            None => 0x07,
        };
        let value = FastOutDiMaxCalGuardChannel{fast_out, max_cal, guard_channel}.as_byte();

        self.write_reg_map_reg(&FoMcGuard, value)?;
        self.register_map.fo_mc_guard.update(value);

        Ok(())
    }

    pub fn set_low_power_mode(&mut self, interval: Duration) -> Result<(), E> {
        let duration = (interval.as_millis() / 8) as u8;
        self.write_reg_map_reg(&LowPowerMode, duration)?;
        self.register_map
            .low_power_mode
            .update(duration);
        Ok(())
    }

    pub fn set_max_on_duration(&mut self, interval: Option<Duration>) -> Result<(), E> {
        let interval = match interval {
            Some(duration) => (duration.as_millis() / 160) as u8,
            None => 0,
        };
        self.write_reg_map_reg(&MaxOnDuration, interval)?;
        self.register_map
            .max_on_duration
            .update(interval);

        Ok(())
    }

    pub fn start_calibrate(&mut self) -> Result<(), E> {
        self.write_reg_map_reg(&Calibrate, 0x01)?;
        self.register_map.calibrate = 0x01;

        Ok(())
    }

    pub fn start_reset(&mut self) -> Result<(), E> {
        self.write_reg_map_reg(&Reset, 0x01)?;
        self.register_map.reset = 0x01;

        Ok(())
    }

    //0
    pub fn read_cached_chip_id(&self) -> (u8, u8) {
        let major_id = self.register_map.chip_id.major_id;
        let minor_id = self.register_map.chip_id.minor_id;

        (major_id, minor_id)
    }

    pub fn read_chip_id(&mut self) -> Result<(u8, u8), E> {
        self.sync_one(&ChipID)?;

        Ok(self.read_cached_chip_id())
    }

    //1
    pub fn read_cached_firmware_version(&self) -> u8 {
        self.register_map.firmware_version
    }

    pub fn read_firmware_version(&mut self) -> Result<u8, E> {
        self.sync_one(&FirmwareVersion)?;

        Ok(self.register_map.firmware_version)
    }

    //2
    pub fn read_cached_detection_status(&self) -> (bool, bool, bool) {
        let calibrate = self.register_map.detection_status.calibrate;
        let overflow = self.register_map.detection_status.overflow;
        let touch = self.register_map.detection_status.touch;

        (calibrate, overflow, touch)
    }

    pub fn read_detection_status(&mut self) -> Result<(bool, bool, bool), E> {
        self.sync_one(&DetectionStatus)?;

        Ok(self.read_cached_detection_status())
    }

    //3
    pub fn read_cached_key_status(&self, key: Key) -> bool {
        let status = &self.register_map.key_status;

        status.key[key as usize]
    }

    pub fn read_key_status(&mut self, key: Key) -> Result<bool, E> {
        self.sync_one(&KeyStatus)?;

        Ok(self.read_cached_key_status(key))
    }

    pub fn read_cached_full_key_status(&self) -> [bool; 7]{
        self.register_map.key_status.key
    }

    pub fn read_full_key_status(&mut self) -> Result<[bool; 7], E>{
        self.sync_one(&KeyStatus)?;

        Ok(self.read_cached_full_key_status())
    }

    //4-17
    pub fn read_cached_key_signal(&self, key: Key) -> u16 {
        let ms = self.register_map.get_key_signal_register(&key, true);
        let ls = self.register_map.get_key_signal_register(&key, false);
        ((*ms as u16) << 8) | (*ls as u16)
    }

    pub fn read_key_signal(&mut self, key: Key) -> Result<u16, E> {
        self.sync_one(&KeySignalMs(key))?;
        self.sync_one(&KeySignalLs(key))?;

        Ok(self.read_cached_key_signal(key))
    }

    //18-31
    pub fn read_cached_reference_data(&self, key: Key) -> u16 {
        let ms = self.register_map.get_reference_data_register(&key, true);
        let ls = self.register_map.get_reference_data_register(&key, false);
        ((*ms as u16) << 8) | (*ls as u16)
    }

    pub fn read_reference_data(&mut self, key: Key) -> Result<u16, E> {
        self.sync_one(&ReferenceDataMs(key))?;
        self.sync_one(&ReferenceDataLs(key))?;

        Ok(self.read_cached_reference_data(key))
    }

    //32-38
    pub fn read_cached_negative_threshold(&self, key: Key) -> u8 {
        *self.register_map.get_nthr_key_register(&key)
    }

    pub fn read_negative_threshold(&mut self, key: Key) -> Result<u8, E> {
        self.sync_one(&NthrKey(key))?;

        Ok(self.read_cached_negative_threshold(key))
    }

    //39-45
    pub fn read_cached_ave_aks(&self, key: Key) -> (u8, u8){
        let ave_aks = self.register_map.get_ave_aks_key_register(&key);
        (ave_aks.ave, ave_aks.aks)
    }

    pub fn read_ave_aks(&mut self, key: Key) -> Result<(u8, u8), E>{
        self.sync_one(&AveAksKey(key))?;

        Ok(self.read_cached_ave_aks(key))
    }

    //46-52
    pub fn read_cached_detection_integrator(&self, key: Key) -> u8 {
        *self.register_map.get_di_key_register(&key)
    }

    pub fn read_detection_integrator(&mut self, key: Key) -> Result<u8, E> {
        self.sync_one(&DIKey(key))?;

        Ok(self.read_cached_detection_integrator(key))
    }

    //53
    pub fn read_cached_fo_mc_guard(&self) -> (bool, bool, u8){
        let fo_mc_guard = &self.register_map.fo_mc_guard;
        (fo_mc_guard.fast_out, fo_mc_guard.max_cal, fo_mc_guard.guard_channel)
    }

    pub fn read_fo_mc_guard(&mut self) -> Result<(bool, bool, u8), E> {
        self.sync_one(&FoMcGuard)?;

        Ok(self.read_cached_fo_mc_guard())
    }

    //54
    pub fn read_cached_low_power_mode(&self) -> Duration {
        let value = self.register_map.low_power_mode.as_byte();
        if value == 0 {
            return Duration::from_millis(8);
        }
        Duration::from_millis(value as u64 * 8)
    }

    pub fn read_low_power_mode(&mut self) -> Result<Duration, E> {
        self.sync_one(&LowPowerMode)?;

        Ok(self.read_cached_low_power_mode())
    }

    //55
    pub fn read_cached_max_on_duration(&self) -> Option<Duration> {
        let value = self.register_map.max_on_duration.as_byte();
        if value == 0 {
            return None;
        }

        Some(Duration::from_millis(value as u64 * 160))
    }

    pub fn read_max_on_duration(&mut self) -> Result<Option<Duration>, E> {
        self.sync_one(&MaxOnDuration)?;

        Ok(self.read_cached_max_on_duration())
    }

    pub fn sync_all(&mut self) -> Result<(), E> {
        let new = self.read_all_reg()?;

        self.register_map
            .chip_id
            .update(new[RegisterMap::get_register_addr(&ChipID) as usize]);
        self.register_map.firmware_version =
            new[RegisterMap::get_register_addr(&FirmwareVersion) as usize];
        self.register_map
            .detection_status
            .update(new[RegisterMap::get_register_addr(&DetectionStatus) as usize]);
        self.register_map
            .key_status
            .update(new[RegisterMap::get_register_addr(&KeyStatus) as usize]);
        for key in 0..7 {
            *self
                .register_map
                .get_key_signal_register_mut(&Key::from(key), true) =
                new[RegisterMap::get_register_addr(&KeySignalMs(Key::from(key))) as usize];
            *self
                .register_map
                .get_key_signal_register_mut(&Key::from(key), false) =
                new[RegisterMap::get_register_addr(&KeySignalLs(Key::from(key))) as usize];
            *self
                .register_map
                .get_reference_data_register_mut(&Key::from(key), true) =
                new[RegisterMap::get_register_addr(&ReferenceDataMs(Key::from(key))) as usize];
            *self
                .register_map
                .get_reference_data_register_mut(&Key::from(key), false) =
                new[RegisterMap::get_register_addr(&ReferenceDataLs(Key::from(key))) as usize];
            *self.register_map.get_nthr_key_register_mut(&Key::from(key)) =
                new[RegisterMap::get_register_addr(&NthrKey(Key::from(key))) as usize];
            self.register_map
                .get_ave_aks_key_register_mut(&Key::from(key))
                .update(new[RegisterMap::get_register_addr(&AveAksKey(Key::from(key))) as usize]);
            *self.register_map.get_di_key_register_mut(&Key::from(key)) =
                new[RegisterMap::get_register_addr(&DIKey(Key::from(key))) as usize];
        }
        self.register_map
            .fo_mc_guard
            .update(new[RegisterMap::get_register_addr(&FoMcGuard) as usize]);
        self.register_map
            .low_power_mode
            .update(new[RegisterMap::get_register_addr(&LowPowerMode) as usize]);
        self.register_map
            .max_on_duration
            .update(new[RegisterMap::get_register_addr(&MaxOnDuration) as usize]);
        self.register_map.calibrate = new[RegisterMap::get_register_addr(&Calibrate) as usize];
        self.register_map.reset = new[RegisterMap::get_register_addr(&Reset) as usize];

        Ok(())
    }

    pub fn sync_one(&mut self, reg: &Register) -> Result<(), E> {
        match reg {
            Register::ChipID => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.chip_id.update(value)
            }
            Register::FirmwareVersion => {
                self.register_map.firmware_version =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::DetectionStatus => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.detection_status.update(value)
            }
            Register::KeyStatus => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.key_status.update(value)
            }
            Register::KeySignalMs(key) => {
                *self.register_map.get_key_signal_register_mut(key, true) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::KeySignalLs(key) => {
                *self.register_map.get_key_signal_register_mut(key, false) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::ReferenceDataMs(key) => {
                *self.register_map.get_reference_data_register_mut(key, true) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::ReferenceDataLs(key) => {
                *self
                    .register_map
                    .get_reference_data_register_mut(key, false) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::NthrKey(key) => {
                *self.register_map.get_nthr_key_register_mut(key) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::AveAksKey(key) => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map
                    .get_ave_aks_key_register_mut(key)
                    .update(value);
            }
            Register::DIKey(key) => {
                *self.register_map.get_di_key_register_mut(key) =
                    self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::FoMcGuard => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.fo_mc_guard.update(value);
            }
            Register::LowPowerMode => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.low_power_mode.update(value);
            }
            Register::MaxOnDuration => {
                let value = self.read_reg(RegisterMap::get_register_addr(reg))?;
                self.register_map.max_on_duration.update(value);
            }
            Register::Calibrate => {
                self.register_map.calibrate = self.read_reg(RegisterMap::get_register_addr(reg))?
            }
            Register::Reset => {
                self.register_map.reset = self.read_reg(RegisterMap::get_register_addr(reg))?
            }
        }

        Ok(())
    }

    fn read_reg(&mut self, register_idx: u8) -> Result<u8, E> {
        if register_idx >= REGISTER_COUNT {
            return Ok(0);
        }

        let mut register_buf = [0u8; 1];
        self.i2c
            .write_read(AT42QT1070_I2C_ADDR, &[register_idx], &mut register_buf)?;

        Ok(register_buf[0])
    }

    fn read_all_reg(&mut self) -> Result<[u8; REGISTER_COUNT as usize], E> {
        let mut register_buf = [0u8; REGISTER_COUNT as usize];
        self.i2c
            .write_read(AT42QT1070_I2C_ADDR, &[0], &mut register_buf)?;

        Ok(register_buf)
    }

    fn write_reg_map_reg(&mut self, reg: &Register, value: u8) -> Result<(), E> {
        match reg {
            ChipID | FirmwareVersion | DetectionStatus | KeyStatus | KeySignalMs(_)
            | KeySignalLs(_) | ReferenceDataMs(_) | ReferenceDataLs(_) => return Ok(()),
            _ => {}
        }

        self.write_reg(RegisterMap::get_register_addr(reg), value)
    }

    fn write_reg(&mut self, reg_addr: u8, value: u8) -> Result<(), E> {
        let reg_buf = [reg_addr, value];
        self.i2c.write(AT42QT1070_I2C_ADDR, &reg_buf)
    }
}