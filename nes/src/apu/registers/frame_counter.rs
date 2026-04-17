// Frame counter ($4017)
bitflags!{
    pub struct FrameCounterRegister: u8{
        const IRQ_INHIBIT   = 0b0100_0000;
        const SEQUENCER_MODE= 0b1000_0000;
    }
}

impl FrameCounterRegister{
    pub fn new() -> Self{
        FrameCounterRegister::empty()
    }

    pub fn irq_inhibit(&self) -> bool{
        self.contains(FrameCounterRegister::IRQ_INHIBIT)
    }

    pub fn sequencer_mode(&self) -> bool{
        self.contains(FrameCounterRegister::SEQUENCER_MODE)
    }

    pub fn update(&mut self, data: u8){
        *self = FrameCounterRegister::from_bits_truncate(data)
            & FrameCounterRegister::from_bits_truncate(0b1100_0000);
    }
}
