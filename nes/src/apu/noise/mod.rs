// @trace-pilot c4adaf1b864408e9402af94b21c40f1e406b87ac
// APU Noise

use registers::control::ControlRegister;
use registers::period::PeriodRegister;
use registers::length::LengthRegister;

pub mod registers;

pub struct NoiseRegister{
    pub ctrl: ControlRegister,
    pub period: PeriodRegister,
    pub length: LengthRegister,
}

impl NoiseRegister{
    pub fn new() -> Self{
        NoiseRegister{
            ctrl: ControlRegister::new(),
            period: PeriodRegister::new(),
            length: LengthRegister::new(),
        }
    }
}

