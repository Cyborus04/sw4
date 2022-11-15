extern "C" {
    pub fn blit(sprite: *const u8, x: i32, y: i32, width: u32, height: u32, flags: u32);
    #[link_name = "blitSub"]
    pub fn blit_sub(
        sprite: *const u8,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        src_x: u32,
        src_y: u32,
        stride: u32,
        flags: u32,
    );
    pub fn line(x1: i32, y1: i32, x2: i32, y2: i32);
    pub fn hline(x: i32, y: i32, len: u32);
    pub fn vline(x: i32, y: i32, len: u32);
    pub fn oval(x: i32, y: i32, width: u32, height: u32);
    pub fn rect(x: i32, y: i32, width: u32, height: u32);
    
    pub fn tone(frequency: u32, duration: u32, volume: u32, flags: u32);

    #[link_name = "textUtf8"]
    pub fn text_utf8(ptr: *const u8, len: usize, x: i32, y: i32);
    
    #[link_name = "traceUtf8"]
    pub fn trace_utf8(ptr: *const u8, len: usize);
}
