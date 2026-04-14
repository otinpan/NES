// @trace-pilot f645e1722e5679530e116931116a6177e19a0828
// timer high
pub struct TimerHighRegister{
    value: u8,
}

impl TimerHighRegister{
    pub fn new() -> Self{
        Self {value:0}
    }

    pub fn timer_high(&self) -> u8{
        self.value & 0b0000_0111
    }

    pub fn length_counter_load(&self) -> u8{
        (self.value >> 3) & 0b0001_1111
    }

    pub fn update(&mut self,data: u8){
        self.value=data;
    }

    pub fn bits(&self) ->u8{
        self.value
    }
}
