#[rustfmt::skip]

// @trace-pilot 67fb2c1c9df0bc696b53f8f34a22649b3ac6937b
// Duty Cycle Sequences
#[rustfmt::skip] 
pub static DUTY_TABLE: [[u8;8];4]=[
    // 12.5%
    [0,1,0,0,0,0,0,0],
    // 25%
    [0,1,1,0,0,0,0,0],
    // 50%
    [0,1,1,1,1,0,0,0],
    // 75%
    [1,0,0,1,1,1,1,1],
];
