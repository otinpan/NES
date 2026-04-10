pub mod frame;
pub mod palette;

use crate::ppu::NesPPU;
use frame::Frame;
use crate::cartridge::Mirroring;
// 指定したタイルが使う背景パレットを取り出す
// // @trace-pilot 0adce2467340933e6ff45d59125eb73d93519fe8
// Each attribute table, starting at $23C0, $27C0, $2BC0, or $2FC0, is arranged as an 8x8 byte array:
fn bg_palette(ppu: &NesPPU,tile_column: usize,tile_row: usize) ->[u8;4]{
    let attr_table_idx=tile_row/4*8+tile_column/4;
    let attr_byte=ppu.vram[0x3c0+attr_table_idx];

    let palette_idx=match (tile_column%4/2,tile_row%4/2){
        (0,0) => attr_byte & 0b11,
        (1,0) => (attr_byte>>2) & 0b11,
        (0,1) => (attr_byte>>4) & 0b11,
        (1,1) => (attr_byte>>6) & 0b11,
        (_,_) => panic!("should not happen"),
    };

    // @trace-pilot 4f1e474b5f86419bd58a815d3b1e4b2340beff5e
    // Backgrounds and sprites each have 4 palettes of 4 colors
    let palette_start: usize=1+(palette_idx as usize)*4;
    [
        ppu.palette_table[0],
        ppu.palette_table[palette_start],
        ppu.palette_table[palette_start+1],
        ppu.palette_table[palette_start+2]
    ]
}


fn sprite_palette(ppu: &NesPPU,palette_idx:u8) -> [u8;4]{
    let start=0x11+(palette_idx*4) as usize;
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start+1],
        ppu.palette_table[start+2]
    ]
}



pub fn render(ppu: &NesPPU,frame:&mut Frame){
    // draw background
    let bank=ppu.ctrl.bknd_pattern_addr();
    // @trace-pilot 89cfaccd775e4f7344d525570d130c664713767b
    // each nametable has 30 rows of 32 tiles each, for 960 ($3C0) bytes
    for i in 0..0x3c0{
        let tile=ppu.vram[i] as u16;
        let tile_column=i%32;
        let tile_row=i/32;
        let tile=&ppu.chr_rom[(bank+tile*16) as usize..=(bank + tile *16 +15) as usize];
        // どのpalette tableを使用するか
        let palette=bg_palette(ppu,tile_column,tile_row);

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
                    0 => palette::SYSTEM_PALETTE[ppu.palette_table[0] as usize],
                    1 => palette::SYSTEM_PALETTE[palette[1] as usize],
                    2 => palette::SYSTEM_PALETTE[palette[2] as usize],
                    3 => palette::SYSTEM_PALETTE[palette[3] as usize],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_column*8+x,tile_row*8+y,rgb)
            }
        }
    }

    // draw sprites
    for i in (0..ppu.oam_data.len()).step_by(4).rev(){
        let tile_idx=ppu.oam_data[i+1] as u16;
        let tile_x=ppu.oam_data[i+3] as usize;
        let tile_y=ppu.oam_data[i] as usize;

        // @trace-pilot 7524d3d6fa8e9efa3106f6e565fe6ef6f2027e2c
        // Byte 2 - Attributes
        let flip_vertical=if ppu.oam_data[i+2]>>7 & 1 ==1{
            true
        }else{
            false
        };

        let flip_horizontal=if ppu.oam_data[i+2]>>6 & 1==1{
            true
        }else{
            false
        };

        let palette_idx=ppu.oam_data[i+2] & 0b11;
        let sprite_palette=sprite_palette(ppu,palette_idx);

        let bank: u16=ppu.ctrl.sprt_pattern_addr();
        let tile=&ppu.chr_rom[(bank+tile_idx*16) as usize..=(bank+tile_idx*16+15) as usize];

        for y in 0..7{
            let mut upper=tile[y];
            let mut lower=tile[y+8];
            'ololo: for x in (0..=7).rev(){
                let value=(1&lower) <<1 | (1&upper);
                upper=upper>>1;
                lower=lower>>1;
                let rgb=match value{
                    0 => continue 'ololo,
                    1 => palette::SYSTEM_PALETTE[sprite_palette[1] as usize],
                    2 => palette::SYSTEM_PALETTE[sprite_palette[2] as usize],
                    3 => palette::SYSTEM_PALETTE[sprite_palette[3] as usize],
                    _ => panic!("can't be"),
                };

                match (flip_horizontal,flip_vertical){
                    (false,false) => frame.set_pixel(tile_x+x,tile_y+y,rgb),
                    (true,false) => frame.set_pixel(tile_x+7-x,tile_y+y,rgb),
                    (false,true) => frame.set_pixel(tile_x+x,tile_y+7-y,rgb),
                    (true,true) => frame.set_pixel(tile_x+7-x,tile_y+7-y,rgb),
                }
            }
        }
    }
}

