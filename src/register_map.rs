use core::borrow::{Borrow, BorrowMut};

pub const REGISTER_COUNT: u8 = 58;

pub trait RegisterMapRegister {
    fn as_byte(&self) -> u8;
    fn update(&mut self, val: u8);
}

pub struct ChipId {
    pub major_id: u8,
    pub minor_id: u8,
}

impl Default for ChipId {
    fn default() -> Self {
        Self {
            major_id: 0x2,
            minor_id: 0xE,
        }
    }
}

impl RegisterMapRegister for ChipId {
    fn as_byte(&self) -> u8 {
        self.major_id << 4 | self.minor_id
    }

    fn update(&mut self, val: u8) {
        self.minor_id = val >> 4;
        self.major_id = val & 0x0F;
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
    pub key: [bool; 7],
}

impl Default for KeyStatus {
    fn default() -> Self {
        Self {
            reserved: false,
            key: [false; 7],
        }
    }
}

impl RegisterMapRegister for KeyStatus {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.reserved {
            r |= 1 << 7;
        }
        for i in 0..7 {
            if self.key[i] {
                r |= 1 << i;
            }
        }

        r
    }

    fn update(&mut self, val: u8) {
        self.reserved = val & 1 << 7 != 0;
        for i in 0..7 {
            self.key[i] = val & 1 << i != 0;
        }
    }
}

#[derive(Copy, Clone)]
pub struct AveAks {
    pub ave: u8,
    pub aks: u8,
}

impl Default for AveAks {
    fn default() -> Self {
        Self {
            ave: 0x08,
            aks: 0x01,
        }
    }
}

impl RegisterMapRegister for AveAks {
    fn as_byte(&self) -> u8 {
        self.ave << 2 | self.aks
    }

    fn update(&mut self, val: u8) {
        self.ave = val >> 2;
        self.aks = val & 0x03;
    }
}

pub struct FastOutDiMaxCalGuardChannel {
    pub fast_out: bool,
    pub max_cal: bool,
    pub guard_channel: u8,
}

impl Default for FastOutDiMaxCalGuardChannel {
    fn default() -> Self {
        Self {
            fast_out: false,
            max_cal: false,
            guard_channel: 0x00,
        }
    }
}

impl RegisterMapRegister for FastOutDiMaxCalGuardChannel {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.fast_out {
            r |= 1 << 5;
        }
        if self.max_cal {
            r |= 1 << 4;
        }

        r | self.guard_channel
    }

    fn update(&mut self, val: u8) {
        self.fast_out = val & 1 << 5 != 0;
        self.max_cal = val & 1 << 4 != 0;
        self.guard_channel = val & 0x0F
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
    pub key_signal_ms: [u8; 7],                   //0x10, 0x0E, 0x0C, 0x0A, 0x08, 0x06, 0x04
    pub key_signal_ls: [u8; 7],                   //0x11, 0x0F, 0x0D, 0x0B, 0x09, 0x07, 0x05
    pub reference_data_ms: [u8; 7],               //0x1E, 0x1C, 0x1A, 0x18, 0x16, 0x14, 0x12
    pub reference_data_ls: [u8; 7],               //0x1F, 0x1D, 0x1B, 0x19, 0x17, 0x15, 0x13
    pub nthr_key: [u8; 7],                        //0x26 to 0x20
    pub ave_aks_key: [AveAks; 7],                 //0x2D to 0x27
    pub di_key: [u8; 7],                          //0x34 to 0x2E
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
            key_signal_ms: [0x00; 7],
            key_signal_ls: [0x00; 7],
            reference_data_ms: [0x00; 7],
            reference_data_ls: [0x00; 7],
            nthr_key: [0x14; 7],
            ave_aks_key: [AveAks::default(); 7],
            di_key: [0x04; 7],
            fo_mc_guard: FastOutDiMaxCalGuardChannel::default(),
            low_power_mode: LowPowerMode::default(),
            max_on_duration: MaxOnDuration::default(),
            calibrate: 0x00,
            reset: 0x00,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Key {
    Key0 = 0,
    Key1 = 1,
    Key2 = 2,
    Key3 = 3,
    Key4 = 4,
    Key5 = 5,
    Key6 = 6,
}

impl From<u8> for Key {
    fn from(val: u8) -> Self {
        let val = val % 7;
        match val {
            0 => Key::Key0,
            1 => Key::Key1,
            2 => Key::Key2,
            3 => Key::Key3,
            4 => Key::Key4,
            5 => Key::Key5,
            6 => Key::Key6,
            _ => panic!(),
        }
    }
}

pub enum Register {
    ChipID,
    FirmwareVersion,
    DetectionStatus,
    KeyStatus,
    KeySignalMs(Key),
    KeySignalLs(Key),
    ReferenceDataMs(Key),
    ReferenceDataLs(Key),
    NthrKey(Key),
    AveAksKey(Key),
    DIKey(Key),
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
            Register::KeySignalMs(key) => *self.get_key_signal_register(key, true),
            Register::KeySignalLs(key) => *self.get_key_signal_register(key, false),
            Register::ReferenceDataMs(key) => *self.get_reference_data_register(key, true),
            Register::ReferenceDataLs(key) => *self.get_reference_data_register(key, false),
            Register::NthrKey(key) => *self.get_nthr_key_register(key),
            Register::AveAksKey(key) => self.get_ave_aks_key_register(&key).as_byte(),
            Register::DIKey(key) => *self.get_di_key_register(key),
            Register::FoMcGuard => self.fo_mc_guard.as_byte(),
            Register::LowPowerMode => self.low_power_mode.as_byte(),
            Register::MaxOnDuration => self.max_on_duration.as_byte(),
            Register::Calibrate => self.calibrate,
            Register::Reset => self.reset,
        }
    }

    pub fn get_key_signal_register_mut(&mut self, key: &Key, ms: bool) -> &mut u8 {
        if ms {
            self.key_signal_ms[*key as usize].borrow_mut()
        } else {
            self.key_signal_ls[*key as usize].borrow_mut()
        }
    }

    pub fn get_key_signal_register(&self, key: &Key, ms: bool) -> &u8 {
        if ms {
            self.key_signal_ms[*key as usize].borrow()
        } else {
            self.key_signal_ls[*key as usize].borrow()
        }
    }

    pub fn get_reference_data_register_mut(&mut self, key: &Key, ms: bool) -> &mut u8 {
        if ms {
            self.reference_data_ms[*key as usize].borrow_mut()
        } else {
            self.reference_data_ls[*key as usize].borrow_mut()
        }
    }

    pub fn get_reference_data_register(&self, key: &Key, ms: bool) -> &u8 {
        if ms {
            self.reference_data_ms[*key as usize].borrow()
        } else {
            self.reference_data_ls[*key as usize].borrow()
        }
    }

    pub fn get_nthr_key_register_mut(&mut self, key: &Key) -> &mut u8 {
        self.nthr_key[*key as usize].borrow_mut()
    }

    pub fn get_nthr_key_register(&self, key: &Key) -> &u8 {
        self.nthr_key[*key as usize].borrow()
    }

    pub fn get_ave_aks_key_register_mut(&mut self, key: &Key) -> &mut AveAks {
        self.ave_aks_key[*key as usize].borrow_mut()
    }

    pub fn get_ave_aks_key_register(&self, key: &Key) -> &AveAks {
        self.ave_aks_key[*key as usize].borrow()
    }

    pub fn get_di_key_register_mut(&mut self, key: &Key) -> &mut u8 {
        self.di_key[*key as usize].borrow_mut()
    }

    pub fn get_di_key_register(&self, key: &Key) -> &u8 {
        self.di_key[*key as usize].borrow()
    }

    pub fn get_register_addr(reg: &Register) -> u8 {
        match reg {
            Register::ChipID => 0x00,
            Register::FirmwareVersion => 0x01,
            Register::DetectionStatus => 0x02,
            Register::KeyStatus => 0x03,
            Register::KeySignalMs(key) => 0x04 + RegisterMap::get_key_register_offset(*key) * 2,
            Register::KeySignalLs(key) => 0x05 + RegisterMap::get_key_register_offset(*key) * 2,
            Register::ReferenceDataMs(key) => 0x12 + RegisterMap::get_key_register_offset(*key) * 2,
            Register::ReferenceDataLs(key) => 0x13 + RegisterMap::get_key_register_offset(*key) * 2,
            Register::NthrKey(key) => 0x20 + RegisterMap::get_key_register_offset(*key),
            Register::AveAksKey(key) => 0x27 + RegisterMap::get_key_register_offset(*key),
            Register::DIKey(key) => 0x2E + RegisterMap::get_key_register_offset(*key),
            Register::FoMcGuard => 0x35,
            Register::LowPowerMode => 0x36,
            Register::MaxOnDuration => 0x37,
            Register::Calibrate => 0x38,
            Register::Reset => 0x39,
        }
    }

    fn get_key_register_offset(key: Key) -> u8 {
        7 - key as u8
    }
}
