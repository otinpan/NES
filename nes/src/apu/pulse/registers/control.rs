// @trace-pilot ed7f1010358b15d0ea2ef002838324d8da4d8e84
// $4000
pub struct ControlRegister {
    value: u8,
}

impl ControlRegister {
    const LENGTH_HALT: u8 = 0b0010_0000;
    const CONSTANT_VOLUME: u8 = 0b0001_0000;

    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn volume(&self) -> u8 {
        self.value & 0b0000_1111
    }

    pub fn duty(&self) -> u8 {
        (self.value >> 6) & 0b11
    }

    pub fn length_halt(&self) -> bool {
        (self.value & Self::LENGTH_HALT) != 0
    }

    pub fn constant_volume(&self) -> bool {
        (self.value & Self::CONSTANT_VOLUME) != 0
    }

    pub fn envelope_period(&self) -> u8 {
        self.value & 0b0000_1111
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }

    pub fn bits(&self) -> u8 {
        self.value
    }
}
