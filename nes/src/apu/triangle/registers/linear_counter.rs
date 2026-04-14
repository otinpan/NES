// @trace-pilot aa7ce2f7dbd4176605f2ad06ef4c0ec607b4034e
// Linear counter

pub struct LinearCounterRegister{
    value: u8,
}

impl LinearCounterRegister{
    pub fn new() -> Self{
        Self{ value: 0}
    }

    pub fn control_flag(&self) -> bool{
        ((self.value>>7) & 0b1) !=0
    }

    pub fn reload(&self) -> u8{
        self.value & 0b0111_1111
    }

    pub fn update(&mut self,data: u8){
        self.value=data;
    }

    pub fn bits(&self) -> u8{
        self.value
    }
}
