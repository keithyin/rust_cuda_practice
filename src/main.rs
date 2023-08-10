use std::fs;
use std::time::Instant;
use std::sync::Arc;

mod image;
mod utils;

use image::Image;

fn main() {

    let content = fs::read("sample_5184_3456.bmp").unwrap();
    let mut width_u8 = [0 as u8; 4];
    let mut height_u8 = [0 as u8; 4];
    
    // vec 直接搞 指针类型转换的话，会报 not aligned 异常。哎，就这么搞了。
    for i in 0 as usize..4 {
        width_u8[i] = content[i + 18 as usize];
    }

    for i in 0 as usize..4 {
        height_u8[i] = content[i + 22 as usize];
    }

    let width = utils::four_u8_to_i32(&width_u8);
    let height: u32 = utils::four_u8_to_i32(&height_u8);

    let size: usize = (3 * width * height) as usize;

    let img_pixels = &content[54..(54+size)];

    let iter_num = 51 as u128;
    let num_threads = 10;

    let mut img = Image::new(&content[0..54], &content[(54+size)..], width, height, 3, img_pixels);
    let mut tot_elapsed = 0;
    for _ in 0..iter_num {
        tot_elapsed += img.flip_vertical_cocurrently(num_threads);
    }
    println!("vertical_time: tot:{:?}, avg:{:?}", tot_elapsed, tot_elapsed / iter_num);
    img.save(String::from("flip_vertical.bmp"));

    
    let mut img = Image::new(&content[0..54], &content[(54+size)..], width, height, 3, img_pixels);
    let mut tot_elapsed = 0;
    for _ in 0..iter_num {
        tot_elapsed += img.flip_vertical_cocurrently_memory_friendly(num_threads);
    }
    println!("vertical_time_memory_friendly: tot:{:?}, avg:{:?}", tot_elapsed, tot_elapsed / iter_num);
    img.save(String::from("flip_vertical_memory_friendly.bmp"));


    let mut img = Image::new(&content[0..54], &content[(54+size)..], width, height, 3, img_pixels);
    
    tot_elapsed = 0 as u128;
    for _ in 0..iter_num {
        tot_elapsed += img.flip_horizontal_cocurrently(num_threads);
    }
    println!("horizontal_time: tot:{:?}, avg:{:?}", tot_elapsed, tot_elapsed / iter_num);
    img.save(String::from("flip_horizontal.bmp"));

    let mut img = Image::new(&content[0..54], &content[(54+size)..], width, height, 3, img_pixels);
    
    tot_elapsed = 0 as u128;
    for _ in 0..iter_num {
        tot_elapsed += img.flip_horizontal_cocurrently_memory_friendly(num_threads);
    }
    println!("horizontal_time_mem_friendly: tot:{:?}, avg:{:?}", tot_elapsed, tot_elapsed / iter_num);
    img.save(String::from("flip_horizontal_mem_friendly.bmp"));


    println!("width:{:?}, height:{:?}", width, height);

    println!("{0}", content[0]);

    println!("Hello, world!");
}
