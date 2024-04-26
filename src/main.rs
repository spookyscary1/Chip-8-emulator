use std::env::args;
use std::fs;


extern crate sdl2;

use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use soloud::Soloud;
use std::time::Duration;
use sdl2::rect::Rect;
use rand::Rng;




const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

fn get_byte(abyte:i32, opcode:u16 ) ->u16 {
    match abyte{
        0=> return opcode >> 12,
        1=> return (opcode << 4) >>12,
        2=> return (opcode <<8 ) >>12,
        3=>return opcode& 0xF,
        _=> return 1,
    }
}
fn get_addr(opcode:u16) -> u16{
    return opcode & 0xFFF;
}
fn get_last_two(opcode:u16) -> u8{
    return (opcode & 0xFF) as u8;
}


struct Chip8 {
    v: [u8;16],
    I: u16,
    SP: u16,
    PC:u16,
    delay: u8,
    sound: u8,
    memory: [u8;4096],
    display:[bool;64*32],
    stack: [u16;16],
    keys: [bool;16],
}
impl Chip8 {
    fn new() -> Chip8{
        return Chip8 {
            SP:0,
            PC: 0x200,
            delay:0,
            sound:0,
            v: [0;16],
            I:0,
            memory:[0;4096],
            display: [false;64*32],
            stack:[0;16],
            keys:[false;16],
        };

    }
    fn push(&mut self, value: u16 ) {
        self.stack[self.SP as usize]=value;
        self.SP+=1;
    }
    fn pop(&mut self) -> u16{
        let address =self.stack[(self.SP-1) as usize];
        self.SP-=1;
        return  address;
    }
    fn load_rom(&mut self, rom: Vec<u8>){
        let mut counter =0;
        self.memory[..80].copy_from_slice(&FONTSET);
        while counter< rom.len(){
           self.memory[self.PC as usize+ counter ]= rom[counter];
           counter+=1;
        }   
    }
    fn update_timers(&mut self, sl: &Soloud, wav: &soloud::Wav ){
        if self.delay > 0{
            self.delay -=1;
        }
        if self.sound > 0 {
            if sl.voice_count()==0{
              sl.play(wav);
            }
            self.sound -=1;
        }
        
    }
    fn fetch_execute(&mut self){
        let opcode=((self.memory[self.PC as usize ] as u16) << 8) | self.memory[(self.PC+1) as usize] as u16;
        match opcode>> 12 {
                    0 =>{ if opcode ==0x00E0  { self.display= [false;64*32];                
                    };
                    if opcode == 0x00EE {
                        self.PC=self.pop()};
                    }
                    1 => {self.PC=get_addr(opcode); self.PC-=2;},
                    2=> {
                        self.push(self.PC);
                        self.PC=get_addr(opcode);
                        self.PC-=2;
                    },
                    3=> {
                        if self.v[get_byte(1, opcode) as usize]==get_last_two(opcode){
                            self.PC+=2;
                        }
                    },
                    4=> {
                        if self.v[get_byte(1, opcode)as usize]!=get_last_two(opcode){
                            self.PC+=2;
                        }
                    },
                    5=> {
                        if self.v[get_byte(1, opcode) as usize] == self.v[get_byte(2, opcode)as usize]{
                            self.PC+=2;
                        }
                    }
                    6=> {
                        self.v[get_byte(1, opcode) as usize]=get_last_two(opcode);
                    },
                    7=> self.v[get_byte(1, opcode) as usize]=(get_last_two(opcode) as u16 +self.v[get_byte(1, opcode) as usize]as u16)as u8,
                    8=>{ match get_byte(3, opcode) {
                        0 => self.v[get_byte(1, opcode)as usize]= self.v[get_byte(2, opcode)as usize],
                        1 => self.v[get_byte(1, opcode)as usize]= self.v[get_byte(1, opcode)as usize] | self.v[get_byte(2, opcode)as usize],
                        2 =>self.v[get_byte(1, opcode)as usize]= self.v[get_byte(1, opcode)as usize] & self.v[get_byte(2, opcode)as usize],
                        3 =>self.v[get_byte(1, opcode)as usize]= self.v[get_byte(1, opcode)as usize] ^ self.v[get_byte(2, opcode)as usize],
                        4=>{
                            let ans= self.v[get_byte(1, opcode)as usize] as u16 + self.v[get_byte(2, opcode)as usize] as u16;
                            self.v[get_byte(1,  opcode) as usize]= ans as u8;
                            if ans >=255{
                                self.v[0xf]=1;
                            }
                            else{
                                self.v[0xf]=0;
                            }
                        },
                        5=>{
                            let orignal_vx = self.v[get_byte(1, opcode) as usize];
                            let orignal_vy= self.v[get_byte(2, opcode) as usize];
                            let answer =  self.v[get_byte(1, opcode) as usize].wrapping_sub(self.v[get_byte(2, opcode) as usize]);
                            self.v[get_byte(1, opcode) as usize]=answer;
                            if orignal_vx>= orignal_vy{
                                self.v[0xf]=1;
                            }
                            else{
                                self.v[0xf]=0;
                            }
                        },
                        
                        6=>{
                            // need to make configurable
                            let orignal_vx= self.v[get_byte(1, opcode) as usize];
                            self.v[get_byte(1, opcode)as usize]=self.v[get_byte(1, opcode)as usize] >>1;
                            self.v[0xf]= ((orignal_vx&0b1) !=0) as u8;
                        },
                        7=>{
                            let orignal_vx = self.v[get_byte(1, opcode) as usize];
                            let orignal_vy= self.v[get_byte(2, opcode) as usize];
                            let answer =  self.v[get_byte(2, opcode) as usize].wrapping_sub(self.v[get_byte(1, opcode) as usize]);
                            self.v[get_byte(1, opcode) as usize]=answer;
                            if orignal_vx<= orignal_vy{
                                self.v[0xf]=1;
                            }
                            else{
                                self.v[0xf]=0;
                            }
                            },
                        
                        0xE=>{
                            // need to make configurable
                            let orignal_vx = self.v[get_byte(1, opcode) as usize];
                            
                            self.v[get_byte(1, opcode)as usize]=self.v[get_byte(1, opcode)as usize] <<1;
                            self.v[0xf]= ((orignal_vx&0b10000000) ==128) as u8;
                        },
                        _=> print!("8")
                    }},
                    9=>{
                        if self.v[get_byte(1, opcode)as usize]!= self.v[get_byte(2, opcode)as usize]{
                            self.PC+=2;
                        }
                    }
                    0xA=> {
                        self.I= get_addr(opcode);
                    },
                    0xB=> {
                        self.PC= self.v[0]as u16+get_addr(opcode) -2;
                    },
                    0xC=> {
                        let mut rng= rand::thread_rng();
                        self.v[get_byte(1, opcode)as usize]= rng.gen::<u8>() & get_last_two(opcode);
                    }
                    0xD =>{
                        self.v[0xf]=0;
                        let x_register_number= get_byte(1, opcode);
                        let y_register_number= get_byte(2, opcode);
                        let x_position= self.v[x_register_number as usize] %64;
                        let y_position = self.v[y_register_number as usize] %32;
                        let rows = get_byte(3, opcode);

                        for sprite_row  in 0..rows  {
                            let sprite_row_data= self.memory[(self.I + sprite_row)as usize];
                            for bit in 0..8{
                                if (x_position+bit < 64) & ((y_position+ sprite_row as u8) < 32){
                                    let index:u16 = (x_position +bit) as u16 +((y_position as u16+sprite_row as u16)*64);
                                    let screen_pixel = self.display[index as usize];
                                    let sprite_pixel = (sprite_row_data & (1 <<7 - bit)) !=0;
                                    if sprite_pixel & screen_pixel{
                                        self.v[0xf] =1;
                                        self.display[index as usize]=false;
                                    }
                                    if sprite_pixel & !screen_pixel{
                                        self.display[index as usize]=true;
                                    } 
                                }
                            }
                            
                        }
                    },
                    0xE=> {match get_last_two(opcode){
                        0xA1=>{
                            let register_value= self.v[get_byte(1, opcode)as usize];
                            if  !self.keys[register_value as usize]{
                                self.PC+=2;
                            }
                        },
                        0x9E => {
                            let register_value= self.v[get_byte(1, opcode)as usize];
                            if  self.keys[register_value as usize]{
                                self.PC+=2;
                            }
                        }, 
                        _ => println!("E instruction:not found"),
                    }},
                    0xF=> {
                        match get_last_two(opcode) {
                            0x7 => {
                                self.v[get_byte(1, opcode) as usize]= self.delay;
                            },
                            0x0a=> if !self.keys.contains(&true){
                                self.PC-=2;
                            }
                            else{
                                for i in 0..15{
                                    if self.keys[i as usize]{
                                        self.v[get_byte(1, opcode)as usize]=i;
                                        break;
                                    }
                                }
                            },
                            0x15 => {
                                self.delay=self.v[get_byte(1, opcode) as usize];
                            },
                            0x18 => self.sound=self.v[get_byte(1, opcode) as usize],
                            0x1e=> self.I+= self.v[get_byte(1, opcode) as usize]as u16,
                            0x29 => {
                                self.I= get_byte(1, opcode) *5;
                            },
                            0x33 => {
                                 let mut number = self.v[get_byte(1, opcode) as usize];
                                 self.memory[self.I as usize+2]=number%10;
                                 number/=10;
                                 self.memory[self.I as usize+1]=number%10;
                                 number/=10;
                                 self.memory[self.I as usize]=number%10;
                            },
                            0x55  => {
                                for i in 0..get_byte(1, opcode)+1{
                                    self.memory[(self.I+i) as usize]=self.v[i as usize];
                                }
                            },
                            0x65=>{
                                for i in 0..get_byte(1, opcode)+1{
                                    self.v[i as usize]= self.memory[(self.I+i) as usize]
                                }
                            },
                            _=> println!("F instruction not found")
                        }
                    },
                    _=> print!("")
                }
        self.PC+=2;
    }
}


pub struct Renderer { 
    canvas: WindowCanvas,
    DOT_SIZE_IN_PXS: u32,
 }
impl Renderer {
    pub fn new(window: Window ) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas,DOT_SIZE_IN_PXS: 20 })
    }
    fn clear(&mut self){
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.present();
    }
    fn draw_screen(&mut self, display: [bool;64*32] ){
        self.canvas.set_draw_color(Color::BLACK);
        self.clear();
        self.canvas.set_draw_color(Color::GREEN);
        let mut counter=0;
        while counter < display.len(){
            if display[counter]{
                let x = counter %64;
                let y = counter /64;
                self.canvas.fill_rect(Rect::new(
                    x as i32* self.DOT_SIZE_IN_PXS as i32 ,
                    y as i32* self.DOT_SIZE_IN_PXS as i32,
                    self.DOT_SIZE_IN_PXS,
                    self.DOT_SIZE_IN_PXS,
                )).expect("");
                
            };
            counter+=1;
            }
            self.canvas.present();
            self.canvas.set_draw_color(Color::BLACK);
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String>= args().collect();
    if args.len()< 2 {
        println!("No file location");
        return  Ok(());
    }
    // intialize hardware 
    let mut Chip8= Chip8::new();
    // loading rom
    let file = fs::read(&args[1]);
    let file = file.expect("File could not be opened");
    Chip8.load_rom(file);


    //graphics intializing
    const GRID_X_SIZE: u32 = 64;
    const GRID_Y_SIZE: u32 = 32;
    const DOT_SIZE_IN_PXS: u32 = 20;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video().expect("video subsystem");
    let window = video_subsystem
    .window(
        "chip 8",
        GRID_X_SIZE * DOT_SIZE_IN_PXS,
        GRID_Y_SIZE * DOT_SIZE_IN_PXS
    )
    .position_centered()
    .opengl()
    .build()
    .map_err(|e| e.to_string()).expect("map");
    let mut gfx = Renderer::new(window).expect("render");

    let mut event_pump = sdl_context.event_pump().expect("event pump");

    // audio setup 
    let sl = Soloud::default().expect("audio");

    let mut wav = <soloud::audio::Wav as soloud::AudioExt>::default();

    soloud::LoadExt::load(&mut wav, &std::path::Path::new("beep.mp3")).expect("audio load");
    //end audio setup 

    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyUp { 
                    keycode: Some(keycode),.. 
                }=> match keycode {
                    Keycode::Num1=> Chip8.keys[1]=false,
                    Keycode::Num2=> Chip8.keys[2]=false,
                    Keycode::Num3=> Chip8.keys[3]=false,
                    Keycode::Num4=> Chip8.keys[0xc]=false,
                    // row two
                    Keycode::Q=> Chip8.keys[4]=false,
                    Keycode::W=> Chip8.keys[5]=false,
                    Keycode::E=> Chip8.keys[6]=false,
                    Keycode::R=> Chip8.keys[0xD]=false,
                    //row three
                    Keycode::A=> Chip8.keys[7]=false,
                    Keycode::S=> Chip8.keys[8]=false,
                    Keycode::D=> Chip8.keys[9]=false,
                    Keycode::F=> Chip8.keys[0xE]=false,
                    //row four
                    Keycode::Z=> Chip8.keys[0xA]=false,
                    Keycode::X=> Chip8.keys[0]=false,
                    Keycode::C=> Chip8.keys[0xB]=false,
                    Keycode::V=> Chip8.keys[0xF]=false,
                    _ => {},
                    
                },
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode{
                    Keycode::Num1=> Chip8.keys[1]=true,
                    Keycode::Num2=> Chip8.keys[2]=true,
                    Keycode::Num3=> Chip8.keys[3]=true,
                    Keycode::Num4=> Chip8.keys[0xC]=true,
                    // row two
                    Keycode::Q=> Chip8.keys[4]=true,
                    Keycode::W=> Chip8.keys[5]=true,
                    Keycode::E=> Chip8.keys[6]=true,
                    Keycode::R=> Chip8.keys[0xD]=true,
                    //row three
                    Keycode::A=> Chip8.keys[7]=true,
                    Keycode::S=> Chip8.keys[8]=true,
                    Keycode::D=> Chip8.keys[9]=true,
                    Keycode::F=> Chip8.keys[0xE]=true,
                    //row four
                    Keycode::Z=> Chip8.keys[0xA]=true,
                    Keycode::X=> Chip8.keys[0]=true,
                    Keycode::C=> Chip8.keys[0xB]=true,
                    Keycode::V=> Chip8.keys[0xF]=true,
                    _=>{}

                }

                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
        Chip8.fetch_execute();
        Chip8.update_timers(&sl, &wav);
        gfx.draw_screen(Chip8.display);
    }

    return Ok(());
    
    
  
}
