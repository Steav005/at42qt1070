#![no_std]
#![allow(dead_code)]

use embedded_hal::blocking::i2c;

mod register_map;
use crate::register_map::Register::*;
use core::time::Duration;
use register_map::*;

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

    pub fn set_negative_threshold(&mut self, threshold: u8, key: u8) -> Result<(), E> {
        if key > 6 {
            return Ok(());
        }

        *self.register_map.get_nthr_key_register_mut(key).unwrap() = threshold;
        self.write_reg_map_reg(&NthrKey(key))
    }

    pub fn set_ave_aks(&mut self) -> Result<(), E> {
        //TODO Figure Out what ave and aks are
        Ok(())
    }

    pub fn set_detection_integrator(&mut self, di: u8, key: u8) -> Result<(), E> {
        if key > 6 {
            return Ok(());
        }

        *self.register_map.get_di_key_register_mut(key).unwrap() = di;
        self.write_reg_map_reg(&DIKey(key))
    }

    pub fn set_fo_mc_guard(&mut self) -> Result<(), E> {
        //TODO And/or split?

        Ok(())
    }

    pub fn set_low_power_mode(&mut self, interval: Duration) -> Result<(), E> {
        self.register_map
            .low_power_mode
            .update((interval.as_millis() / 8) as u8);
        self.write_reg_map_reg(&LowPowerMode)
    }

    pub fn set_max_on_duration(&mut self, interval: Duration) -> Result<(), E> {
        self.register_map
            .max_on_duration
            .update((interval.as_millis() / 160) as u8);
        self.write_reg_map_reg(&MaxOnDuration)
    }

    pub fn start_calibrate(&mut self) -> Result<(), E> {
        self.register_map.calibrate = 0x01;
        self.write_reg_map_reg(&Calibrate)
    }

    pub fn start_reset(&mut self) -> Result<(), E> {
        self.register_map.reset = 0x01;
        self.write_reg_map_reg(&Reset)
    }

    pub fn read_cached_chip_id(&self) -> (u8, u8) {
        let major_id = self.register_map.chip_id.major_id;
        let minor_id = self.register_map.chip_id.minor_id;

        (u8::from(major_id), u8::from(minor_id))
    }

    pub fn read_chip_id(&mut self) -> Result<(u8, u8), E> {
        self.read_reg(RegisterMap::get_register_index(&ChipID))?;

        Ok(self.read_cached_chip_id())
    }

    pub fn read_cached_firmware_version(&self) -> u8 {
        self.register_map.firmware_version
    }

    pub fn read_firmware_version(&mut self) -> Result<u8, E> {
        self.read_reg(RegisterMap::get_register_index(&FirmwareVersion))?;

        Ok(self.register_map.firmware_version)
    }

    pub fn read_cached_detection_status(&self) -> (bool, bool, bool) {
        let calibrate = self.register_map.detection_status.calibrate;
        let overflow = self.register_map.detection_status.overflow;
        let touch = self.register_map.detection_status.touch;

        (calibrate, overflow, touch)
    }

    pub fn read_detection_status(&mut self) -> Result<(bool, bool, bool), E> {
        self.read_reg(RegisterMap::get_register_index(&DetectionStatus))?;

        Ok(self.read_cached_detection_status())
    }

    pub fn read_cached_calibrating_status(&self) -> bool {
        self.register_map.detection_status.calibrate
    }

    pub fn read_calibrating_status(&mut self) -> Result<bool, E> {
        let (calibrate, _, _) = self.read_detection_status()?;

        Ok(calibrate)
    }

    pub fn read_cached_overflow_status(&self) -> bool {
        self.register_map.detection_status.overflow
    }

    pub fn read_overflow_status(&mut self) -> Result<bool, E> {
        let (_, overflow, _) = self.read_detection_status()?;

        Ok(overflow)
    }

    pub fn read_cached_touch_status(&self) -> bool {
        self.register_map.detection_status.touch
    }

    pub fn read_touch_status(&mut self) -> Result<bool, E> {
        let (_, _, touch) = self.read_detection_status()?;

        Ok(touch)
    }

    pub fn read_cached_key_status(&self, key: u8) -> bool {
        let status = &self.register_map.key_status;

        match key {
            0 => status.key0,
            1 => status.key1,
            2 => status.key2,
            3 => status.key3,
            4 => status.key4,
            5 => status.key5,
            6 => status.key6,
            _ => false,
        }
    }

    pub fn read_key_status(&mut self, key: u8) -> Result<bool, E> {
        self.read_reg(RegisterMap::get_register_index(&KeyStatus))?;

        Ok(self.read_cached_key_status(key))
    }

    //TODO Continue with Register Address 0x04

    pub fn sync_all(&mut self) -> Result<(), E> {
        let new = self.read_all_reg()?;

        self.register_map
            .chip_id
            .update(new[RegisterMap::get_register_index(&ChipID) as usize]);
        self.register_map.firmware_version =
            new[RegisterMap::get_register_index(&FirmwareVersion) as usize];
        self.register_map
            .detection_status
            .update(new[RegisterMap::get_register_index(&DetectionStatus) as usize]);
        self.register_map
            .key_status
            .update(new[RegisterMap::get_register_index(&KeyStatus) as usize]);
        for key in 0..7 {
            *self
                .register_map
                .get_key_signal_register_mut(key, true)
                .unwrap() = new[RegisterMap::get_register_index(&KeySignalMs(key)) as usize];
            *self
                .register_map
                .get_key_signal_register_mut(key, false)
                .unwrap() = new[RegisterMap::get_register_index(&KeySignalLs(key)) as usize];
            *self
                .register_map
                .get_reference_data_register_mut(key, true)
                .unwrap() = new[RegisterMap::get_register_index(&ReferenceDataMs(key)) as usize];
            *self
                .register_map
                .get_reference_data_register_mut(key, false)
                .unwrap() = new[RegisterMap::get_register_index(&ReferenceDataLs(key)) as usize];
            *self.register_map.get_nthr_key_register_mut(key).unwrap() =
                new[RegisterMap::get_register_index(&NthrKey(key)) as usize];
            self.register_map
                .get_ave_aks_key_register_mut(key)
                .unwrap()
                .update(new[RegisterMap::get_register_index(&AveAksKey(key)) as usize]);
            *self.register_map.get_di_key_register_mut(key).unwrap() =
                new[RegisterMap::get_register_index(&DIKey(key)) as usize];
        }
        self.register_map
            .fo_mc_guard
            .update(new[RegisterMap::get_register_index(&FoMcGuard) as usize]);
        self.register_map
            .low_power_mode
            .update(new[RegisterMap::get_register_index(&LowPowerMode) as usize]);
        self.register_map
            .max_on_duration
            .update(new[RegisterMap::get_register_index(&MaxOnDuration) as usize]);
        self.register_map.calibrate = new[RegisterMap::get_register_index(&Calibrate) as usize];
        self.register_map.reset = new[RegisterMap::get_register_index(&Reset) as usize];

        Ok(())
    }

    pub fn sync_one(&mut self, reg: &Register) -> Result<(), E> {
        match reg {
            Register::ChipID => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.chip_id.update(value)
            }
            Register::FirmwareVersion => {
                self.register_map.firmware_version =
                    self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::DetectionStatus => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.detection_status.update(value)
            }
            Register::KeyStatus => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.key_status.update(value)
            }
            Register::KeySignalMs(key) => {
                *self
                    .register_map
                    .get_key_signal_register_mut(key.clone(), true)
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::KeySignalLs(key) => {
                *self
                    .register_map
                    .get_key_signal_register_mut(key.clone(), false)
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::ReferenceDataMs(key) => {
                *self
                    .register_map
                    .get_reference_data_register_mut(key.clone(), true)
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::ReferenceDataLs(key) => {
                *self
                    .register_map
                    .get_reference_data_register_mut(key.clone(), false)
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::NthrKey(key) => {
                *self
                    .register_map
                    .get_nthr_key_register_mut(key.clone())
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::AveAksKey(key) => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map
                    .get_ave_aks_key_register_mut(key.clone())
                    .unwrap()
                    .update(value);
            }
            Register::DIKey(key) => {
                *self
                    .register_map
                    .get_di_key_register_mut(key.clone())
                    .unwrap() = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::FoMcGuard => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.fo_mc_guard.update(value);
            }
            Register::LowPowerMode => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.low_power_mode.update(value);
            }
            Register::MaxOnDuration => {
                let value = self.read_reg(RegisterMap::get_register_index(reg))?;
                self.register_map.max_on_duration.update(value);
            }
            Register::Calibrate => {
                self.register_map.calibrate = self.read_reg(RegisterMap::get_register_index(reg))?
            }
            Register::Reset => {
                self.register_map.reset = self.read_reg(RegisterMap::get_register_index(reg))?
            }
        }

        Ok(())
    }

    fn read_reg(&mut self, register_idx: u8) -> Result<u8, E> {
        if register_idx < 1 || register_idx > REGISTER_COUNT {
            return Ok(0);
        }

        let mut register_buf = [0u8; 1];
        self.i2c
            .try_write_read(AT42QT1070_I2C_ADDR, &[register_idx], &mut register_buf)?;

        Ok(register_buf[0])
    }

    fn read_all_reg(&mut self) -> Result<[u8; REGISTER_COUNT as usize], E> {
        let mut register_buf = [0u8; REGISTER_COUNT as usize];
        self.i2c
            .try_write_read(AT42QT1070_I2C_ADDR, &[0], &mut register_buf)?;

        Ok(register_buf)
    }

    fn write_reg_map_reg(&mut self, reg: &Register) -> Result<(), E> {
        match reg {
            ChipID | FirmwareVersion | DetectionStatus | KeyStatus | KeySignalMs(_)
            | KeySignalLs(_) | ReferenceDataMs(_) | ReferenceDataLs(_) => return Ok(()),
            _ => {}
        }

        let value = self.register_map.reg_as_byte(reg);
        self.write_reg(RegisterMap::get_register_index(reg), value)
    }

    fn write_reg(&mut self, reg_addr: u8, value: u8) -> Result<(), E> {
        let reg_buf = [reg_addr, value];
        self.i2c.try_write(AT42QT1070_I2C_ADDR, &reg_buf)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
