pub mod frame;
pub mod pallete;

use crate::ppu::NesPPU;
use frame::Frame;

pub fn render(ppu: &NesPPU,frame:&mut Frame){
    let bank=ppu.ctrl.bknd_pattern_addr();
    // @trace-pilot 89cfaccd775e4f7344d525570d130c664713767b
    // each nametable has 30 rows of 32 tiles each, for 960 ($3C0) bytes
    for i in 0..0x3c0{
        let tile=ppu.vram[i] as u16;
        let tile_column=i%32;
        let tile_row=i/32;
        let tile=&ppu.chr_rom[(bank+tile*16) as usize..=(bank + tile *16 +15) as usize];
        // @trace-pilot c3c949a866040711e4a57cbcca7fff1f02e6259e
        // Each byte in the nametable controls one 8x8 pixel character cell
        for y in 0..=7{
            // @trace-pilot 765c9db25e943b7a38ba855cbb16c5f3eff5853d
            // the low and then high bitplane of the pattern data for that tile ID
            let mut upper=tile[y];
            let mut lower=tile[y+8];

            for x in (0..=7).rev(){
                let value=(1&lower) <<1 | (1 & upper);
                upper=upper>>1;
                lower=lower>>1;
                let rgb=match value{
                    0 => pallete::SYSTEM_PALLETE[0x01],
                    1 => pallete::SYSTEM_PALLETE[0x23],
                    2 => pallete::SYSTEM_PALLETE[0x27],
                    3 => pallete::SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_column*8+x,tile_row*8+y,rgb)
            }
        }
    }
}
