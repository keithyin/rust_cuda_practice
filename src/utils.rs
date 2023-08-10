


pub fn four_u8_to_i32(data: &[u8]) -> u32 {
    let u8_ptr = data.as_ptr();
    let i32_ptr = u8_ptr as *const u32;
    return unsafe {
        *i32_ptr
    };
}