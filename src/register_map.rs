use core::borrow::{Borrow, BorrowMut};
use ux::{u2, u4, u6};

pub const REGISTER_COUNT: u8 = 58;

pub trait RegisterMapRegister {
    fn as_byte(&self) -> u8;
    fn update(&mut self, val: u8);
}

pub struct ChipId {
    pub major_id: u4,
    pub minor_id: u4,
}

impl Default for ChipId {
    fn default() -> Self {
        Self {
            major_id: u4::new(0x2),
            minor_id: u4::new(0xE),
        }
    }
}

impl RegisterMapRegister for ChipId {
    fn as_byte(&self) -> u8 {
        u8::from(self.major_id) << 4 | u8::from(self.minor_id)
    }

    fn update(&mut self, val: u8) {
        self.minor_id = u4::new(val >> 4);
        self.major_id = u4::new(val & 0x0F);
    }
}

pub struct DetectionStatus {
    pub calibrate: bool,
    pub overflow: bool,
    pub touch: bool,
}

impl Default for DetectionStatus {
    fn default() -> Self {
        Self {
            calibrate: false,
            overflow: false,
            touch: false,
        }
    }
}

impl RegisterMapRegister for DetectionStatus {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.calibrate {
            r |= 1 << 7;
        }
        if self.overflow {
            r |= 1 << 6;
        }
        if self.touch {
            r |= 1;
        }

        r
    }

    fn update(&mut self, val: u8) {
        self.calibrate = val & 1 << 7 != 0;
        self.overflow = val & 1 << 6 != 0;
        self.touch = val & 1 != 0;
    }
}

pub struct KeyStatus {
    pub reserved: bool,
    pub key6: bool,
    pub key5: bool,
    pub key4: bool,
    pub key3: bool,
    pub key2: bool,
    pub key1: bool,
    pub key0: bool,
}

impl Default for KeyStatus {
    fn default() -> Self {
        Self {
            reserved: false,
            key6: false,
            key5: false,
            key4: false,
            key3: false,
            key2: false,
            key1: false,
            key0: false,
        }
    }
}

impl RegisterMapRegister for KeyStatus {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.reserved {
            r |= 1 << 7;
        }
        if self.key6 {
            r |= 1 << 6;
        }
        if self.key5 {
            r |= 1 << 5;
        }
        if self.key4 {
            r |= 1 << 4;
        }
        if self.key3 {
            r |= 1 << 3;
        }
        if self.key2 {
            r |= 1 << 2;
        }
        if self.key1 {
            r |= 1 << 1;
        }
        if self.key0 {
            r |= 1;
        }

        r
    }

    fn update(&mut self, val: u8) {
        self.reserved = val & 1 << 7 != 0;
        self.key6 = val & 1 << 6 != 0;
        self.key5 = val & 1 << 5 != 0;
        self.key4 = val & 1 << 4 != 0;
        self.key3 = val & 1 << 3 != 0;
        self.key2 = val & 1 << 2 != 0;
        self.key1 = val & 1 << 1 != 0;
        self.key0 = val & 1 != 0;
    }
}

pub struct AveAks {
    pub ave: u6,
    pub aks: u2,
}

impl Default for AveAks {
    fn default() -> Self {
        Self {
            ave: u6::new(8),
            aks: u2::new(1),
        }
    }
}

impl RegisterMapRegister for AveAks {
    fn as_byte(&self) -> u8 {
        u8::from(self.ave) << 2 | u8::from(self.aks)
    }

    fn update(&mut self, val: u8) {
        self.ave = u6::new(val >> 2);
        self.aks = u2::new(val & 0x03);
    }
}

pub struct FastOutDiMaxCalGuardChannel {
    pub fo: bool,
    pub max_cal: bool,
    pub guard_channel: u4,
}

impl Default for FastOutDiMaxCalGuardChannel {
    fn default() -> Self {
        Self {
            fo: false,
            max_cal: false,
            guard_channel: u4::new(0),
        }
    }
}

impl RegisterMapRegister for FastOutDiMaxCalGuardChannel {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.fo {
            r |= 1 << 5;
        }
        if self.max_cal {
            r |= 1 << 4;
        }

        r | u8::from(self.guard_channel)
    }

    fn update(&mut self, val: u8) {
        self.fo = val & 1 << 5 != 0;
        self.max_cal = val & 1 << 4 != 0;
        self.guard_channel = u4::new(val & 0x0F)
    }
}

pub struct LowPowerMode(u8);

impl Default for LowPowerMode {
    fn default() -> Self {
        Self { 0: 2 }
    }
}

impl RegisterMapRegister for LowPowerMode {
    fn as_byte(&self) -> u8 {
        self.0
    }

    fn update(&mut self, val: u8) {
        self.0 = val;
    }
}

impl LowPowerMode {
    pub fn from_millis(millis: u16) -> Self {
        Self {
            0: (millis / 8) as u8,
        }
    }
}

pub struct MaxOnDuration(u8);

impl Default for MaxOnDuration {
    fn default() -> Self {
        Self { 0: 180 }
    }
}

impl RegisterMapRegister for MaxOnDuration {
    fn as_byte(&self) -> u8 {
        self.0
    }

    fn update(&mut self, val: u8) {
        self.0 = val;
    }
}

impl MaxOnDuration {
    pub fn from_millis(millis: u16) -> Self {
        Self {
            0: (millis / 160) as u8,
        }
    }
}

pub struct RegisterMap {
    pub chip_id: ChipId,                          //0x00
    pub firmware_version: u8,                     //0x01
    pub detection_status: DetectionStatus,        //0x02
    pub key_status: KeyStatus,                    //0x03
    pub key_signal_0_ms: u8,                      //0x04
    pub key_signal_0_ls: u8,                      //0x05
    pub key_signal_1_ms: u8,                      //0x06
    pub key_signal_1_ls: u8,                      //0x07
    pub key_signal_2_ms: u8,                      //0x08
    pub key_signal_2_ls: u8,                      //0x09
    pub key_signal_3_ms: u8,                      //0x0A
    pub key_signal_3_ls: u8,                      //0x0B
    pub key_signal_4_ms: u8,                      //0x0C
    pub key_signal_4_ls: u8,                      //0x0D
    pub key_signal_5_ms: u8,                      //0x0E
    pub key_signal_5_ls: u8,                      //0x0F
    pub key_signal_6_ms: u8,                      //0x10
    pub key_signal_6_ls: u8,                      //0x11
    pub reference_data_0_ms: u8,                  //0x12
    pub reference_data_0_ls: u8,                  //0x13
    pub reference_data_1_ms: u8,                  //0x14
    pub reference_data_1_ls: u8,                  //0x15
    pub reference_data_2_ms: u8,                  //0x16
    pub reference_data_2_ls: u8,                  //0x17
    pub reference_data_3_ms: u8,                  //0x18
    pub reference_data_3_ls: u8,                  //0x19
    pub reference_data_4_ms: u8,                  //0x1A
    pub reference_data_4_ls: u8,                  //0x1B
    pub reference_data_5_ms: u8,                  //0x1C
    pub reference_data_5_ls: u8,                  //0x1D
    pub reference_data_6_ms: u8,                  //0x1E
    pub reference_data_6_ls: u8,                  //0x1F
    pub nthr_key_0: u8,                           //0x20
    pub nthr_key_1: u8,                           //0x21
    pub nthr_key_2: u8,                           //0x22
    pub nthr_key_3: u8,                           //0x23
    pub nthr_key_4: u8,                           //0x24
    pub nthr_key_5: u8,                           //0x25
    pub nthr_key_6: u8,                           //0x26
    pub ave_aks_key_0: AveAks,                    //0x27
    pub ave_aks_key_1: AveAks,                    //0x28
    pub ave_aks_key_2: AveAks,                    //0x29
    pub ave_aks_key_3: AveAks,                    //0x2A
    pub ave_aks_key_4: AveAks,                    //0x2B
    pub ave_aks_key_5: AveAks,                    //0x2C
    pub ave_aks_key_6: AveAks,                    //0x2D
    pub di_key_0: u8,                             //0x2E
    pub di_key_1: u8,                             //0x2F
    pub di_key_2: u8,                             //0x30
    pub di_key_3: u8,                             //0x31
    pub di_key_4: u8,                             //0x32
    pub di_key_5: u8,                             //0x33
    pub di_key_6: u8,                             //0x34
    pub fo_mc_guard: FastOutDiMaxCalGuardChannel, //0x35
    pub low_power_mode: LowPowerMode,             //0x36
    pub max_on_duration: MaxOnDuration,           //0x37
    pub calibrate: u8,                            //0x38
    pub reset: u8,                                //0x39
}

impl Default for RegisterMap {
    fn default() -> Self {
        Self {
            chip_id: ChipId::default(),
            firmware_version: 0x15,
            detection_status: DetectionStatus::default(),
            key_status: KeyStatus::default(),
            key_signal_0_ms: 0x00,
            key_signal_0_ls: 0x00,
            key_signal_1_ms: 0x00,
            key_signal_1_ls: 0x00,
            key_signal_2_ms: 0x00,
            key_signal_2_ls: 0x00,
            key_signal_3_ms: 0x00,
            key_signal_3_ls: 0x00,
            key_signal_4_ms: 0x00,
            key_signal_4_ls: 0x00,
            key_signal_5_ms: 0x00,
            key_signal_5_ls: 0x00,
            key_signal_6_ms: 0x00,
            key_signal_6_ls: 0x00,
            reference_data_0_ms: 0x00,
            reference_data_0_ls: 0x00,
            reference_data_1_ms: 0x00,
            reference_data_1_ls: 0x00,
            reference_data_2_ms: 0x00,
            reference_data_2_ls: 0x00,
            reference_data_3_ms: 0x00,
            reference_data_3_ls: 0x00,
            reference_data_4_ms: 0x00,
            reference_data_4_ls: 0x00,
            reference_data_5_ms: 0x00,
            reference_data_5_ls: 0x00,
            reference_data_6_ms: 0x00,
            reference_data_6_ls: 0x00,
            nthr_key_0: 0x14,
            nthr_key_1: 0x14,
            nthr_key_2: 0x14,
            nthr_key_3: 0x14,
            nthr_key_4: 0x14,
            nthr_key_5: 0x14,
            nthr_key_6: 0x14,
            ave_aks_key_0: AveAks::default(),
            ave_aks_key_1: AveAks::default(),
            ave_aks_key_2: AveAks::default(),
            ave_aks_key_3: AveAks::default(),
            ave_aks_key_4: AveAks::default(),
            ave_aks_key_5: AveAks::default(),
            ave_aks_key_6: AveAks::default(),
            di_key_0: 0x04,
            di_key_1: 0x04,
            di_key_2: 0x04,
            di_key_3: 0x04,
            di_key_4: 0x04,
            di_key_5: 0x04,
            di_key_6: 0x04,
            fo_mc_guard: FastOutDiMaxCalGuardChannel::default(),
            low_power_mode: LowPowerMode::default(),
            max_on_duration: MaxOnDuration::default(),
            calibrate: 0x00,
            reset: 0x00,
        }
    }
}

pub enum Register {
    ChipID,
    FirmwareVersion,
    DetectionStatus,
    KeyStatus,
    KeySignalMs(u8),
    KeySignalLs(u8),
    ReferenceDataMs(u8),
    ReferenceDataLs(u8),
    NthrKey(u8),
    AveAksKey(u8),
    DIKey(u8),
    FoMcGuard,
    LowPowerMode,
    MaxOnDuration,
    Calibrate,
    Reset,
}

impl RegisterMap {
    pub fn reg_as_byte(&self, reg: &Register) -> u8 {
        match reg {
            Register::ChipID => self.chip_id.as_byte(),
            Register::FirmwareVersion => self.firmware_version,
            Register::DetectionStatus => self.detection_status.as_byte(),
            Register::KeyStatus => self.key_status.as_byte(),
            Register::KeySignalMs(key) => *self.get_key_signal_register(key.clone(), true).unwrap(),
            Register::KeySignalLs(key) => {
                *self.get_key_signal_register(key.clone(), false).unwrap()
            }
            Register::ReferenceDataMs(key) => {
                *self.get_reference_data_register(key.clone(), true).unwrap()
            }
            Register::ReferenceDataLs(key) => *self
                .get_reference_data_register(key.clone(), false)
                .unwrap(),
            Register::NthrKey(key) => *self.get_nthr_key_register(key.clone()).unwrap(),
            Register::AveAksKey(key) => self
                .get_ave_aks_key_register(key.clone())
                .unwrap()
                .as_byte(),
            Register::DIKey(key) => *self.get_di_key_register(key.clone()).unwrap(),
            Register::FoMcGuard => self.fo_mc_guard.as_byte(),
            Register::LowPowerMode => self.low_power_mode.as_byte(),
            Register::MaxOnDuration => self.max_on_duration.as_byte(),
            Register::Calibrate => self.calibrate,
            Register::Reset => self.reset,
        }
    }

    pub fn get_key_signal_register_mut(&mut self, key: u8, ms: bool) -> Option<&mut u8> {
        match key {
            0 => {
                if ms {
                    Some(self.key_signal_0_ms.borrow_mut())
                } else {
                    Some(self.key_signal_0_ls.borrow_mut())
                }
            }
            1 => {
                if ms {
                    Some(self.key_signal_1_ms.borrow_mut())
                } else {
                    Some(self.key_signal_1_ls.borrow_mut())
                }
            }
            2 => {
                if ms {
                    Some(self.key_signal_2_ms.borrow_mut())
                } else {
                    Some(self.key_signal_2_ls.borrow_mut())
                }
            }
            3 => {
                if ms {
                    Some(self.key_signal_3_ms.borrow_mut())
                } else {
                    Some(self.key_signal_3_ls.borrow_mut())
                }
            }
            4 => {
                if ms {
                    Some(self.key_signal_4_ms.borrow_mut())
                } else {
                    Some(self.key_signal_4_ls.borrow_mut())
                }
            }
            5 => {
                if ms {
                    Some(self.key_signal_5_ms.borrow_mut())
                } else {
                    Some(self.key_signal_5_ls.borrow_mut())
                }
            }
            6 => {
                if ms {
                    Some(self.key_signal_6_ms.borrow_mut())
                } else {
                    Some(self.key_signal_6_ls.borrow_mut())
                }
            }
            _ => None,
        }
    }

    pub fn get_key_signal_register(&self, key: u8, ms: bool) -> Option<&u8> {
        match key {
            0 => {
                if ms {
                    Some(self.key_signal_0_ms.borrow())
                } else {
                    Some(self.key_signal_0_ls.borrow())
                }
            }
            1 => {
                if ms {
                    Some(self.key_signal_1_ms.borrow())
                } else {
                    Some(self.key_signal_1_ls.borrow())
                }
            }
            2 => {
                if ms {
                    Some(self.key_signal_2_ms.borrow())
                } else {
                    Some(self.key_signal_2_ls.borrow())
                }
            }
            3 => {
                if ms {
                    Some(self.key_signal_3_ms.borrow())
                } else {
                    Some(self.key_signal_3_ls.borrow())
                }
            }
            4 => {
                if ms {
                    Some(self.key_signal_4_ms.borrow())
                } else {
                    Some(self.key_signal_4_ls.borrow())
                }
            }
            5 => {
                if ms {
                    Some(self.key_signal_5_ms.borrow())
                } else {
                    Some(self.key_signal_5_ls.borrow())
                }
            }
            6 => {
                if ms {
                    Some(self.key_signal_6_ms.borrow())
                } else {
                    Some(self.key_signal_6_ls.borrow())
                }
            }
            _ => None,
        }
    }

    pub fn get_reference_data_register_mut(&mut self, key: u8, ms: bool) -> Option<&mut u8> {
        match key {
            0 => {
                if ms {
                    Some(self.reference_data_0_ms.borrow_mut())
                } else {
                    Some(self.reference_data_0_ls.borrow_mut())
                }
            }
            1 => {
                if ms {
                    Some(self.reference_data_1_ms.borrow_mut())
                } else {
                    Some(self.reference_data_1_ls.borrow_mut())
                }
            }
            2 => {
                if ms {
                    Some(self.reference_data_2_ms.borrow_mut())
                } else {
                    Some(self.reference_data_2_ls.borrow_mut())
                }
            }
            3 => {
                if ms {
                    Some(self.reference_data_3_ms.borrow_mut())
                } else {
                    Some(self.reference_data_3_ls.borrow_mut())
                }
            }
            4 => {
                if ms {
                    Some(self.reference_data_4_ms.borrow_mut())
                } else {
                    Some(self.reference_data_4_ls.borrow_mut())
                }
            }
            5 => {
                if ms {
                    Some(self.reference_data_5_ms.borrow_mut())
                } else {
                    Some(self.reference_data_5_ls.borrow_mut())
                }
            }
            6 => {
                if ms {
                    Some(self.reference_data_6_ms.borrow_mut())
                } else {
                    Some(self.reference_data_6_ls.borrow_mut())
                }
            }
            _ => None,
        }
    }

    pub fn get_reference_data_register(&self, key: u8, ms: bool) -> Option<&u8> {
        match key {
            0 => {
                if ms {
                    Some(self.reference_data_0_ms.borrow())
                } else {
                    Some(self.reference_data_0_ls.borrow())
                }
            }
            1 => {
                if ms {
                    Some(self.reference_data_1_ms.borrow())
                } else {
                    Some(self.reference_data_1_ls.borrow())
                }
            }
            2 => {
                if ms {
                    Some(self.reference_data_2_ms.borrow())
                } else {
                    Some(self.reference_data_2_ls.borrow())
                }
            }
            3 => {
                if ms {
                    Some(self.reference_data_3_ms.borrow())
                } else {
                    Some(self.reference_data_3_ls.borrow())
                }
            }
            4 => {
                if ms {
                    Some(self.reference_data_4_ms.borrow())
                } else {
                    Some(self.reference_data_4_ls.borrow())
                }
            }
            5 => {
                if ms {
                    Some(self.reference_data_5_ms.borrow())
                } else {
                    Some(self.reference_data_5_ls.borrow())
                }
            }
            6 => {
                if ms {
                    Some(self.reference_data_6_ms.borrow())
                } else {
                    Some(self.reference_data_6_ls.borrow())
                }
            }
            _ => None,
        }
    }

    pub fn get_nthr_key_register_mut(&mut self, key: u8) -> Option<&mut u8> {
        match key {
            0 => Some(self.nthr_key_0.borrow_mut()),
            1 => Some(self.nthr_key_1.borrow_mut()),
            2 => Some(self.nthr_key_2.borrow_mut()),
            3 => Some(self.nthr_key_3.borrow_mut()),
            4 => Some(self.nthr_key_4.borrow_mut()),
            5 => Some(self.nthr_key_5.borrow_mut()),
            6 => Some(self.nthr_key_6.borrow_mut()),
            _ => None,
        }
    }

    pub fn get_nthr_key_register(&self, key: u8) -> Option<&u8> {
        match key {
            0 => Some(self.nthr_key_0.borrow()),
            1 => Some(self.nthr_key_1.borrow()),
            2 => Some(self.nthr_key_2.borrow()),
            3 => Some(self.nthr_key_3.borrow()),
            4 => Some(self.nthr_key_4.borrow()),
            5 => Some(self.nthr_key_5.borrow()),
            6 => Some(self.nthr_key_6.borrow()),
            _ => None,
        }
    }

    pub fn get_ave_aks_key_register_mut(&mut self, key: u8) -> Option<&mut AveAks> {
        match key {
            0 => Some(self.ave_aks_key_0.borrow_mut()),
            1 => Some(self.ave_aks_key_1.borrow_mut()),
            2 => Some(self.ave_aks_key_2.borrow_mut()),
            3 => Some(self.ave_aks_key_3.borrow_mut()),
            4 => Some(self.ave_aks_key_4.borrow_mut()),
            5 => Some(self.ave_aks_key_5.borrow_mut()),
            6 => Some(self.ave_aks_key_6.borrow_mut()),
            _ => None,
        }
    }

    pub fn get_ave_aks_key_register(&self, key: u8) -> Option<&AveAks> {
        match key {
            0 => Some(self.ave_aks_key_0.borrow()),
            1 => Some(self.ave_aks_key_1.borrow()),
            2 => Some(self.ave_aks_key_2.borrow()),
            3 => Some(self.ave_aks_key_3.borrow()),
            4 => Some(self.ave_aks_key_4.borrow()),
            5 => Some(self.ave_aks_key_5.borrow()),
            6 => Some(self.ave_aks_key_6.borrow()),
            _ => None,
        }
    }

    pub fn get_di_key_register_mut(&mut self, key: u8) -> Option<&mut u8> {
        match key {
            0 => Some(self.di_key_0.borrow_mut()),
            1 => Some(self.di_key_1.borrow_mut()),
            2 => Some(self.di_key_2.borrow_mut()),
            3 => Some(self.di_key_3.borrow_mut()),
            4 => Some(self.di_key_4.borrow_mut()),
            5 => Some(self.di_key_5.borrow_mut()),
            6 => Some(self.di_key_6.borrow_mut()),
            _ => None,
        }
    }

    pub fn get_di_key_register(&self, key: u8) -> Option<&u8> {
        match key {
            0 => Some(self.di_key_0.borrow()),
            1 => Some(self.di_key_1.borrow()),
            2 => Some(self.di_key_2.borrow()),
            3 => Some(self.di_key_3.borrow()),
            4 => Some(self.di_key_4.borrow()),
            5 => Some(self.di_key_5.borrow()),
            6 => Some(self.di_key_6.borrow()),
            _ => None,
        }
    }

    pub fn get_register_index(reg: &Register) -> u8 {
        match reg {
            Register::ChipID => 0x00,
            Register::FirmwareVersion => 0x01,
            Register::DetectionStatus => 0x02,
            Register::KeyStatus => 0x03,
            Register::KeySignalMs(key) => 0x04 + key * 2,
            Register::KeySignalLs(key) => 0x05 + key * 2,
            Register::ReferenceDataMs(key) => 0x12 + key * 2,
            Register::ReferenceDataLs(key) => 0x13 + key * 2,
            Register::NthrKey(key) => 0x20 + key,
            Register::AveAksKey(key) => 0x27 + key,
            Register::DIKey(key) => 0x2E + key,
            Register::FoMcGuard => 0x35,
            Register::LowPowerMode => 0x36,
            Register::MaxOnDuration => 0x37,
            Register::Calibrate => 0x38,
            Register::Reset => 0x39,
        }
    }
}
