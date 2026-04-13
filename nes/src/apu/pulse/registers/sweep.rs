// @trace-pilot ffd3827aa01c13697579241024001ca200088fbe
// APU Sweep
bitflags!{
    // $4001: Pulse sweep register
    //
    // 7  bit  0
    // ---- ----
    // EPPP NSSS
    // |||| ||||
    // |||| |+++- Shift count (0-7)
    // |||| +---- Negate flag
    // ||||       (0: increase frequency, 1: decrease frequency)
    // ||++------ Sweep period
    // |          (how often to apply sweep)
    // +--------- Sweep enable
    //           (0: off, 1: on)
    pub struct SweepRegister: u8{
        const ENABLE            =0b1000_0000;
        const NEGATE_FLAG       =0b0000_1000;
    }
}

impl SweepRegister{
    pub fn new() -> Self{
        SweepRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn sweep_period(&self) -> u8{
        (self.bits()>>4) & 0b0000_0111
    }

    pub fn shift_count(&self) -> u8{
        self.bits() & 0b0000_0111
    }

    pub fn enable(&self) -> bool{
        self.contains(SweepRegister::ENABLE)
    }

    pub fn negate_flag(&self) -> bool{
        self.contains(SweepRegister::NEGATE_FLAG)
    }

    pub fn update(&mut self,data:u8){
        *self=SweepRegister::from_bits_truncate(data);
    }
}
