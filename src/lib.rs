#![no_std]

//! # `sw4`
//! A `struct`-based way to write wasm4 programs
//! 
//! Everything is accessed through the [`Wasm4`] type

#[cfg(not(target_arch = "wasm32"))]
compile_error!("`sw4` is only supported on `wasm32`");

pub use sw4_macros::*;

use core::fmt::Write;

mod raw_api;

const _SIZE_ASSERTIONS: () = {
    use core::mem::size_of;
    assert!(size_of::<Color>() == 4);
    assert!(size_of::<Palette>() == 16);
    assert!(size_of::<DrawColors>() == 2);
    assert!(size_of::<[Gamepad; 4]>() == 4);
    assert!(size_of::<Mouse>() == 5);
    assert!(size_of::<SystemFlags>() == 1);
    assert!(size_of::<Netplay>() == 1);
    assert!(size_of::<FrameBuffer>() == 6400);
    assert!(size_of::<Wasm4>() + 4 == 6560);
};

/// The game state
#[repr(C)]
pub struct Wasm4 {
    pub palette: Palette,
    pub draw_colors: DrawColors,
    pub gamepads: [Gamepad; 4],
    pub mouse: Mouse,
    pub system_flags: SystemFlags,
    pub netplay: Netplay,
    reserved: [u8; 127],
    pub frame_buffer: FrameBuffer,
    pub sounds: SoundSystem,
    pub disk: Disk,
}

/// The game's color palette
#[repr(C)]
pub struct Palette {
    pub a: Color,
    pub b: Color,
    pub c: Color,
    pub d: Color,
}

// `align(4)` to ensure size_of<Color> = 4
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
} 

impl Color {
    pub const BLACK: Self = Self::from_u32(0x000000);
    pub const RED: Self = Self::from_u32(0xFF0000);
    pub const GREEN: Self = Self::from_u32(0x00FF00);
    pub const BLUE: Self = Self::from_u32(0x0000FF);
    pub const CYAN: Self = Self::from_u32(0x00FFFF);
    pub const MAGENTA: Self = Self::from_u32(0xFF00FF);
    pub const YELLOW: Self = Self::from_u32(0xFFFF00);
    pub const WHITE: Self = Self::from_u32(0xFFFFFF);

    pub const fn from_u32(x: u32) -> Self {
        let [b, g, r, _] = x.to_le_bytes();
        Self { r, g, b }
    }

    pub const fn to_u32(self) -> u32 {
        u32::from_le_bytes([self.b, self.g, self.r, 0])
    }
}

#[repr(C)]
pub struct DrawColors(u16);

#[repr(u16)]
pub enum DrawColor {
    Transparent = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
}

impl DrawColors {
    pub fn set_all(&mut self, c1: DrawColor, c2: DrawColor, c3: DrawColor, c4: DrawColor) {
        self.0 = (c1 as u16) | ((c2 as u16) << 4) | ((c3 as u16) << 8) | ((c4 as u16) << 12)
    }

    pub fn set_1(&mut self, color: DrawColor) {
        self.0 = (self.0 & 0b1111_1111_1111_0000) | (color as u16);
    }

    pub fn set_2(&mut self, color: DrawColor) {
        self.0 = (self.0 & 0b1111_1111_0000_1111) | ((color as u16) << 4);
    }

    pub fn set_3(&mut self, color: DrawColor) {
        self.0 = (self.0 & 0b1111_0000_1111_1111) | ((color as u16) << 8);
    }

    pub fn set_4(&mut self, color: DrawColor) {
        self.0 = (self.0 & 0b0000_1111_1111_1111) | ((color as u16) << 12);
    }
}

#[repr(C)]
pub struct Gamepad(u8);

impl Gamepad {
    /// Is the x button pressed?
    pub fn x(&self) -> bool {
        self.0 & 0b0000_0001 != 0
    }
    /// Is the z button pressed?
    pub fn z(&self) -> bool {
        self.0 & 0b0000_0010 != 0
    }

    /// Is the left button pressed?
    pub fn left(&self) -> bool {
        self.0 & 0b0001_0000 != 0
    }
    /// Is the right button pressed?
    pub fn right(&self) -> bool {
        self.0 & 0b0010_0000 != 0
    }
    /// Is the up button pressed?
    pub fn up(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
    /// Is the down button pressed?
    pub fn down(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
}

#[repr(C)]
pub struct Mouse {
    // Since i16 has an alignment of 2, using it here would make `Mouse` also 
    // have an alignment of 2.
    // size_of::<T> % align_of::<T> = 0 must hold, so the size of `Mouse` gets
    // rounded up to 6 from 5, throwing off the rest of the Wasm4 struct layout, 
    // which has to exactly match the expected memory map.
    // So, store it as two bytes each instead, which have an alignment of 1.
    // This does still seem to compile to an `i32.load16_s` instruction. 
    x: [u8; 2],
    y: [u8; 2],
    buttons: u8
}

impl Mouse {
    /// The X coordinate of the mouse cursor
    pub fn x(&self) -> i16 {
        i16::from_le_bytes(self.x)
    }
    /// The Y coordinate of the mouse cursor
    pub fn y(&self) -> i16 {
        i16::from_le_bytes(self.y)
    }
    /// Is the left mouse button pressed?
    pub fn left(&self) -> bool {
        self.buttons & 0b001 != 0
    }
    /// Is the right mouse button pressed?
    pub fn right(&self) -> bool {
        self.buttons & 0b010 != 0
    }
    /// Is the middle mouse button pressed?
    pub fn middle(&self) -> bool {
        self.buttons & 0b100 != 0
    }
}

#[repr(C)]
pub struct SystemFlags(u8);

impl SystemFlags {
    /// Set whether the frame buffer should be kept between frames
    pub fn preserve_framebuffer(&mut self, b: bool) {
        self.0 = (self.0 & 0b10) | (b as u8);
    }
    /// Set whether to hide the gamepad overlay on mobile
    pub fn hide_gamepad_overlay(&mut self, b: bool) {
        self.0 = (self.0 & 0b1) | ((b as u8) << 1);
    }
}

#[repr(C)]
/// Netplay info
pub struct Netplay(u8);

impl Netplay {
    /// The client's player index, in the range 0 to 3
    /// 
    /// This corresponds to the index into the `gamepads` array for the local player
    pub fn player_idx(&self) -> u8 {
        self.0 & 0b11
    }

    /// Is netplay enabled?
    pub fn enabled(&self) -> bool {
        (self.0 & 0b100) != 0
    }
}

#[repr(C)]
pub struct FrameBuffer {
    buf: [u8; (160 * 160) / 4],
}

impl FrameBuffer {
    /// Draw a sprite to the screen
    pub fn sprite(
        &mut self,
        sprite: &[u8],
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        flags: SpriteFlags,
    ) {
        assert(
            (width * height) as usize <= sprite.len() * ((((!flags.0 & 1) + 1) * 4) as usize),
            "not enough sprite data"
        );
        unsafe { raw_api::blit(sprite.as_ptr(), x, y, width, height, flags.0) }
    }

    /// Draw a part of a sprite to the screen
    #[allow(clippy::too_many_arguments)]
    pub fn sub_sprite(
        &mut self,
        sprite: &[u8],
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        src_x: u32,
        src_y: u32,
        stride: u32,
        flags: SpriteFlags,
    ) {
        assert(
            ((width + src_x) + ((height + src_y) * stride)) as usize
                <= sprite.len() * ((((!flags.0 & 1) + 1) * 4) as usize),
            "not enough sprite data"
        );
        unsafe {
            raw_api::blit_sub(
                sprite.as_ptr(),
                x,
                y,
                width,
                height,
                src_x,
                src_y,
                stride,
                flags.0,
            )
        }
    }

    /// Draw a pixel onto the screen
    /// 
    /// Draw color 1 is used for the pixel color
    pub fn pixel(&mut self, x: i32, y: i32) {
        let color = unsafe { (0x14 as *const u8).read() } & 0b1111;
        if color == 0 {
            return;
        }
        let color = (color - 1) & 0b11;
        let idx = (y as usize * 40) + (x as usize >> 2);
        let shift = (x as u8 & 0b11) << 1;
        let mask = !(0b11 << shift);
        self.buf[idx] = (color << shift) | (self.buf[idx] & !mask);
    }

    /// Draw a line onto the screen
    /// 
    /// Draw color 1 is used for the line color
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        unsafe { raw_api::line(x1, y1, x2, y2) }
    }

    /// Draw a horizontal line onto the screen
    /// 
    /// Draw color 1 is used for the line color
    pub fn hline(&mut self, x: i32, y: i32, len: u32) {
        unsafe { raw_api::hline(x, y, len) }
    }

    /// Draw a vertical line onto the screen
    /// 
    /// Draw color 1 is used for the line's color
    pub fn vline(&mut self, x: i32, y: i32, len: u32) {
        unsafe { raw_api::vline(x, y, len) }
    }

    /// Draw an oval onto the screen
    /// 
    /// Draw color 1 is used for the fill color, draw color 2 is used for the 
    /// outline color
    pub fn oval(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe { raw_api::oval(x, y, width, height) }
    }

    /// Draw a rectangle onto the screen
    /// 
    /// Draw color 1 is used for the fill color, draw color 2 is used for the 
    /// outline color
    pub fn rect(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe { raw_api::rect(x, y, width, height) }
    }

    /// Draw text to the screen
    /// 
    /// Draw color 1 is used for the text, Draw color 2 is used for the 
    /// background
    pub fn text(&mut self, s: &str, x: i32, y: i32) {
        unsafe { raw_api::text_utf8(s.as_ptr(), s.len(), x, y) }
    }

    /// Draw formatted text to the screen
    pub fn text_fmt(&mut self, args: core::fmt::Arguments<'_>, x: i32, y: i32) {
        use core::fmt;

        struct TextWriter(i32, i32);

        impl fmt::Write for TextWriter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                unsafe { raw_api::text_utf8(s.as_ptr(), s.len(), self.0, self.1) }
                self.0 += s.len() as i32 * 8;
                Ok(())
            }
        }

        let _ = TextWriter(x, y).write_fmt(args);
    }

}

/// Sprite render flags
pub struct SpriteFlags(u32);

impl SpriteFlags {
    /// Sprite data is in a 1-bit-per-pixel format
    pub const ONE_BPP: Self = Self(0b0000);
    /// Sprite data is in a 2-bit-per-pixel format
    pub const TWO_BPP: Self = Self(0b0001);
    /// Flip the sprite horizontally
    pub const FLIP_X: Self = Self(0b0010);
    /// Flip the sprite vertically
    pub const FLIP_Y: Self = Self(0b0100);
    /// Rotate the sprite 90 degrees counter-clockwise
    pub const ROTATE: Self = Self(0b1000);
}

impl core::ops::BitOr for SpriteFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub struct SoundSystem {
    _a: (),
}

impl SoundSystem {
    pub fn play(&self, sound: Sound) {
        let Sound { start_freq, end_freq, attack, decay, sustain, release, peak_vol, sustain_vol, channel } = sound;
        let frequency = (start_freq as u32) | ((end_freq as u32) << 16);
        let duration = u32::from_le_bytes([sustain, release, decay, attack]);
        let volume = u32::from_le_bytes([sustain_vol, peak_vol, 0, 0]);
        let flags = channel.to_num();
        unsafe { raw_api::tone(frequency, duration, volume, flags) }
    }
}

pub struct Sound {
    pub start_freq: u16,
    pub end_freq: u16,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
    pub peak_vol: u8,
    pub sustain_vol: u8,
    pub channel: Channel,
}

#[derive(Clone, Copy)]
pub enum Channel {
    Pulse1(DutyCycle),
    Pulse2(DutyCycle),
    Triangle,
    Noise,
}

impl Channel {
    const fn to_num(self) -> u32 {
        match self {
            Channel::Pulse1(dc) => (dc as u32) << 2,
            Channel::Pulse2(dc) => ((dc as u32) << 2) | 1,
            Channel::Triangle => 2,
            Channel::Noise => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum DutyCycle {
    Eighth = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
}

pub struct Disk(());

impl Disk {
    pub fn read(&self, buf: &mut [u8]) {
        unsafe { raw_api::diskr(buf.as_mut_ptr(), buf.len()) }
    }
    
    pub fn write(&self, buf: &[u8]) {
        unsafe { raw_api::diskw(buf.as_ptr(), buf.len()) }
    }
}

pub fn trace(s: &str) {
    unsafe { raw_api::trace_utf8(s.as_ptr(), s.len()) }
}

pub fn panic(s: &str) -> ! {
    trace(s);
    core::arch::wasm32::unreachable()
}

pub fn assert(x: bool, s: &str) {
    if !x {
        panic(s)
    }
}

#[panic_handler]
#[cfg(all(not(test), feature = "panic_handler"))] // To quiet RA
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    panic("panicked via core")
}

#[doc(hidden)]
#[deprecated(note = "implementation detail, do not use")]
pub struct SyncUnsafeCell<T>(core::cell::UnsafeCell<T>);

#[allow(deprecated)]
unsafe impl<T> Sync for SyncUnsafeCell<T> where T: Sync {}

#[allow(deprecated)]
impl<T> SyncUnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self(core::cell::UnsafeCell::new(value))
    }

    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}
