// @trace-pilot b6d6835dfc16672840e61fcf4b251453a9eaa7ee
// APU Frame Counter

// @trace-pilot da75ae087c45a33c2cc1dd24729439571523eab8
// mode 0:    mode 1:       function 
// ---------  -----------  -----------------------------  
// - - - f    - - - - -    IRQ (if bit 6 is clear)  
// - l - l    - l - - l    Length counter and sweep  
// e e e e    e e e - e    Envelope and linear counter
bitflags!{
    pub struct FrameCounterRegister: u8{
        const IRQ_INHIBIT   = 0b0100_0000;
        const FIVE_STEP_MODE= 0b1000_0000;
    }
}

impl FrameCounterRegister{
    pub fn new() -> Self{
        FrameCounterRegister::empty()
    }

    pub fn irq_inhibit(&self) -> bool{
        self.contains(FrameCounterRegister::IRQ_INHIBIT)
    }

    pub fn five_step_mode(&self) -> bool{
        self.contains(FrameCounterRegister::FIVE_STEP_MODE)
    }

    pub fn update(&mut self, data: u8){
        *self = FrameCounterRegister::from_bits_truncate(data)
            & FrameCounterRegister::from_bits_truncate(0b1100_0000);
    }
}
