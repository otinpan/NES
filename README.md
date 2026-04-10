### CPU
```
//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|
```

### PPU
```
//  _______________ $4000  _______________
// | Mirrors       |       |               |
// | $0000-$3FFF   |       |               |
// |_ _ _ _ _ _ _ _| $3F20 |               |
// | Palette RAM   |       | Palette RAM   |
// |_______________| $3F00 |_______________|
// | Mirrors       |       |               |
// | $2000-$2EFF   |       |               |
// |_ _ _ _ _ _ _ _| $3000 |               |
// | Attribute Tbl |       | Attribute Tbl |
// |_ _ _ _ _ _ _ _|       |               |
// | Name Table 3  |       | Name Table 1  |
// |_ _ _ _ _ _ _ _| $2800 |_______________|
// | Attribute Tbl |       | Attribute Tbl |
// |_ _ _ _ _ _ _ _|       |               |
// | Name Table 2  |       | Name Table 0  |
// |_ _ _ _ _ _ _ _| $2000 |_______________|
// | Pattern Table |       | Pattern Table |
// | (Sprites)     |       | (Background)  |
// |_ _ _ _ _ _ _ _| $1000 |               |
// | Pattern Table |       |               |
// |               |       |               |
// |_______________| $0000 |_______________|
```

### 描画の流れ
$2000から$2fffまでにname tableとattribute tableが入っている。画面には256x240のピクセルが並んでいる。そして、描画単位は8x8ピクセルのタイルで分割されている。すなわち、256/8(32)*240/8(30)=960のタイルがある。64bytesはattirbute tableでpallete tableのアドレスが入る。960のタイルは4x4ずつ纏められる。ということは32/4*30/4=8x8ブロック存在するということである。各ブロックにはどの色を使用するかを決めるように1byteのattribute tableのアドレスを記述する。$3f00から$4000にはpallete tableがある。ここには描画に使う色がの情報が入っている。$0000から$2000までにはpattern tableがあり、そこにアクセスすることで、カートリッジの画像を得ることが出来る。
まずこのように各タイルにどの画像を置くのかを設定する
```
VRAM[0x2000] (name table)=0x20 (patern table)
VRAM[0x2001]=0x03
...
VRAM[0x23fb]=0x08
```
次に、4x4のタイル集合を1つのブロックとする。さらにその中に入った16個のタイルを2x2で分割した4つのタイルの集合をmeta-tilesという。attribute tableには2x2のmeta-tilesにどのpallete tableが使われるかを指定する。これは{00,01,10,11}の4パターンある。1ブロックには4つのmeta-tilesが含まれるため、ちょうど4つのmeta-tilesにどのpallete tableが使用されるかを設定できる。  
pallete tableはbackgroundとspritesの2種類あり、それぞれ4個設定できる。1つのpallete tableには4つの色を設定できる。そのため
```
VRAM[0x23fc] (attribute table)=0b10_01_11_00
VRAM[0x23fd]=0b11_11_10_11
...
VRAM[0x2fff]=0b00_01_10_10
```
と書いた場合、
```
blocks[0][0]=pallete_table[background][2]
blocks[0][1]=pallete_table[background][1]
...
blocks[959][3]=pallete_table[background][2]
```
といったように、pallete_tableを指定できる。  

各pallete tableには4つの色までしか設定できない。$0x000から$0x1fffには8x8ピクセルの各ピクセルがどの色を持っているか記述されている。これは2bitであり、4種類の色から選ぶということ。
```
VRAM[0x000]=01
VRAM[0x001]=11
...
```
