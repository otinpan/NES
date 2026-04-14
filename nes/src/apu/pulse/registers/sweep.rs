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

// @trace-pilot 97fe1ccecc09eef5cb02224865633d53791b718e
// 自動で周波数（音の高さ）を変える仕組み
impl SweepRegister{
    pub fn new() -> Self{
        SweepRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn period(&self) -> u8{
        (self.bits()>>4) & 0b0000_0111
    }

    pub fn shift(&self) -> u8{
        self.bits() & 0b0000_0111
    }

    pub fn enable(&self) -> bool{
        self.contains(SweepRegister::ENABLE)
    }

    pub fn negate(&self) -> bool{
        self.contains(SweepRegister::NEGATE_FLAG)
    }

    pub fn update(&mut self,data:u8){
        *self=SweepRegister::from_bits_truncate(data);
    }
}
