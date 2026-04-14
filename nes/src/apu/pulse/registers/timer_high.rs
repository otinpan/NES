// @trace-pilot c3acd46e30cd94d05173d9b1251e5cdbad77c572
// $4003/$4007
pub struct TimerHighRegister {
    value: u8,
}

impl TimerHighRegister {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn timer_high(&self) -> u8 {
        self.value & 0b0000_0111
    }

    pub fn length_counter_load(&self) -> u8 {
        (self.value >> 3) & 0b0001_1111
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }

    pub fn bits(&self) -> u8 {
        self.value
    }
}
