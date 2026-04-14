// @trace-pilot a7eb406485512d80b1551418f210e56eee4a42a0
// $400E
pub struct PeriodRegister{
    value: u8,
}

impl PeriodRegister{
    const MODE:u8= 0b1000_0000;

    pub fn new() -> Self{
        Self{value: 0}
    }

    pub fn mode(&self) -> bool{
        (self.value & Self::MODE)!=0
    }

    pub fn period(&self) -> u8{
        self.value&0b0000_1111
    }

    pub fn update(&mut self,data: u8){
        self.value=data;
    }

    pub fn bits(&self) ->u8{
        self.value
    }

}
