// @trace-pilot 56afebf0a30ccc9fa4f00ca410a460ab52e3f3df
// $400F
pub struct LengthRegister{
    value: u8,
}

impl LengthRegister{
    pub fn new() -> Self{
        Self {value:0}
    }

    pub fn length_counter_load(&self) ->u8{
        (self.value >>3) & 0b0001_1111
    }

    pub fn update(&mut self,data: u8){
        self.value=data;
    }

    pub fn bits(&self) -> u8{
        self.value
    }
}
