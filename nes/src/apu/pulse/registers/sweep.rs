// @trace-pilot ffd3827aa01c13697579241024001ca200088fbe
// APU Sweep
pub struct SweepRegister {
    value: u8,
}

impl SweepRegister {
    const ENABLE: u8 = 0b1000_0000;
    const NEGATE_FLAG: u8 = 0b0000_1000;

    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn period(&self) -> u8 {
        (self.value >> 4) & 0b0000_0111
    }

    pub fn shift(&self) -> u8 {
        self.value & 0b0000_0111
    }

    pub fn enable(&self) -> bool {
        (self.value & Self::ENABLE) != 0
    }

    pub fn negate(&self) -> bool {
        (self.value & Self::NEGATE_FLAG) != 0
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }

    pub fn bits(&self) -> u8 {
        self.value
    }
}
