
use std::{fs, vec};
use std::time::Instant;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std;

use std::marker::Send;

#[derive(Debug)]
struct SendableU8Pointer(*mut u8);
unsafe impl Send for SendableU8Pointer {
    
}

unsafe fn exchange_two_pixels(data: *mut u8, position: (u32, u32, u32, u32), width: u32, height: u32, channel: u32) {
    let (row1, col1, row2, col2) = position;
    let pos1 = (channel * (row1 * width + col1)) as usize;
    let pos2 = (channel * (row2 * width + col2)) as usize;
    let data_len = (width * height * channel) as usize;
    if (pos1 + 3) >= data_len || (pos2 + 3) >= data_len {
        return;
    }

    let tmp_r = *data.add(pos1);
    let tmp_g = *data.add(pos1 + 1);
    let tmp_b = *data.add(pos1 + 2);

    *data.add(pos1) = *data.add(pos2);
    *data.add(pos1 + 1) = *data.add(pos2 + 1);
    *data.add(pos1 + 2) = *data.add(pos2 + 2);
    
    *data.add(pos2) = tmp_r;
    *data.add(pos2 + 1) = tmp_g;
    *data.add(pos2 + 2) = tmp_b;
}

pub struct Image {
    header: Vec<u8>,
    tail: Vec<u8>,
    width: u32,
    height: u32,
    channel: u32,
    data: Vec<u8>
}

impl Image {
    pub fn new(header: &[u8], tail: &[u8], width: u32, height: u32, channel: u32, data: &[u8]) -> Self {
        let header = Vec::from(header);
        let tail = Vec::from(tail);
        Image{
            header,
            tail,
            width,
            height,
            channel,
            data: Vec::from(data)
        }
    }

    pub fn exchange_two_pixels(&mut self, position: (u32, u32, u32, u32)) {
        let (row1, col1, row2, col2) = position;
        let pos1 = self.channel * (row1 * self.width + col1);
        let pos2 = self.channel * (row2 * self.width + col2);
        if (pos1 + 3) >= self.data.len() as u32 || (pos2 + 3) >= self.data.len() as u32 {
            return;
        }

        let tmp_r = self.data[pos1 as usize];
        let tmp_g = self.data[(pos1 + 1) as usize];
        let tmp_b = self.data[(pos1 + 2) as usize];

        self.data[pos1 as usize] = self.data[pos2 as usize];
        self.data[(pos1 + 1) as usize] = self.data[(pos2 + 1) as usize];
        self.data[(pos1 + 2) as usize] = self.data[(pos2 + 2) as usize];
        
        self.data[pos2 as usize] = tmp_r;
        self.data[(pos2 + 1) as usize] = tmp_g;
        self.data[(pos2 + 2) as usize] = tmp_b;

    }


    pub fn flip_vertical(&mut self) -> u128{
        // flip vertical. bottom to up
        let now = Instant::now();
        for col in 0..self.width {
            for row in 0..(self.height / 2) {
                let exchange_row = self.height - row;
                self.exchange_two_pixels((row, col, exchange_row, col));
            }
        }
        return now.elapsed().as_micros();
    }

    pub fn flip_vertical_cocurrently(&mut self, num_threads: usize) -> u128{
        // flip vertical. bottom to up
        let mut handles = vec![];
        let width = self.width as usize;
        let height = self.height as usize;

        let num_cols_per_thread = width / num_threads;

        let now = Instant::now();
        for thread_idx in 0..num_threads {
            let data_ptr = SendableU8Pointer(self.data.as_mut_ptr());
            let handle = thread::spawn(move || {
                let _ = &data_ptr;
                let ptr = data_ptr.0;

                let begin_col = thread_idx * num_cols_per_thread;
                let mut end_col = begin_col + num_cols_per_thread;
                if thread_idx == (num_threads - 1) {
                    end_col = width;
                }

                for col in begin_col..end_col {
                    for row in 0..(height / 2) {
                        let exchange_row = height - row;
                        unsafe {
                            exchange_two_pixels(ptr, (row as u32, col as u32, exchange_row as u32, col as u32), 
                                width as u32, height as u32, 3);
    
                        }
    
                    }
                }

            });
            handles.push(handle);


        }

        for handle in handles {
            handle.join().unwrap();
        }
        return now.elapsed().as_micros();
    }

    pub fn flip_vertical_cocurrently_memory_friendly(&mut self, num_threads: usize) -> u128{
        // flip vertical. bottom to up
        // 这个的加速非常明显！！！
        let mut handles = vec![];
        let width = self.width as usize;
        let height = self.height as usize;

        let num_rows_per_thread: usize = height / 2 / num_threads;

        let now = Instant::now();
        for thread_idx in 0..num_threads {
            let data_ptr = SendableU8Pointer(self.data.as_mut_ptr());
            let handle = thread::spawn(move || {
                let _ = &data_ptr;
                let ptr = data_ptr.0;
                let begin_row = thread_idx * num_rows_per_thread;
                let mut end_row = begin_row + num_rows_per_thread;
                if thread_idx == (num_threads - 1) {
                    end_row = height / 2;
                }

                for row in begin_row..end_row {
                    
                    let exchange_row: usize = height - row - 1;
                    let span = width * 3;
                    let mut buf1 = vec![0 as u8; span];
                    let mut buf2 = vec![0 as u8; span];
                    let shift1 = row * width * 3;
                    let shift2 = exchange_row * width * 3;
                    unsafe {
                        // https://stackoverflow.com/questions/66609964/looking-for-a-c-memcpy-equivalent
                        std::ptr::copy_nonoverlapping(ptr.add(shift1), buf1.as_mut_ptr(), span);
                        std::ptr::copy_nonoverlapping(ptr.add(shift2), buf2.as_mut_ptr(), span);
                        std::ptr::copy_nonoverlapping(buf1.as_ptr(), ptr.add(shift2), span);
                        std::ptr::copy_nonoverlapping(buf2.as_ptr(), ptr.add(shift1), span);
                    }

                }

            });
            handles.push(handle);


        }

        for handle in handles {
            handle.join().unwrap();
        }
        return now.elapsed().as_micros();
    }

    pub fn flip_horizontal(&mut self) -> u128{
        // flip horizontal. left to right
        let now = Instant::now();
        for row in 0..self.height {
            for col in 0..(self.width / 2) {
                let excange_col = self.width - col;
                self.exchange_two_pixels((row, col, row, excange_col));
            }
        }
        return now.elapsed().as_micros();
    }

    pub fn flip_horizontal_cocurrently(&mut self, num_threads: usize) -> u128{
        let width = self.width as usize;
        let height = self.height as usize;
        let num_row_per_thread = height / num_threads; // TODO: 这里应该是向上取整的

        let mut handles = vec![];
        let now  = Instant::now();

        for thread_idx in 0..num_threads {
            let data_ptr = SendableU8Pointer(self.data.as_mut_ptr());

            let handle = thread::spawn(move || {
                let _ = &data_ptr;
                let ptr = data_ptr.0;

                let begin_row = thread_idx * num_row_per_thread;
                let mut end_row = (thread_idx + 1) * num_row_per_thread;
                if thread_idx == (num_threads - 1) {
                    end_row = height;
                }
                
                for row in begin_row..end_row {
                    for col in 0..(width / 2) {
                        let excange_col = width - col;
                        unsafe {
                            exchange_two_pixels(ptr, (row as u32, col as u32, row as u32, excange_col as u32), width as u32, height as u32, 3);

                        }
                    }
                }

            });

            handles.push(handle);
            
        }
   
        for handle in handles {
            handle.join().unwrap();
        }
        
        return now.elapsed().as_micros();
        


    }

    pub fn flip_horizontal_cocurrently_memory_friendly(&mut self, num_threads: usize) -> u128{
        // 这个加速不是很明显，甚至还有点慢
        let width = self.width as usize;
        let height = self.height as usize;
        let num_row_per_thread = height / num_threads; // TODO: 这里应该是向上取整的

        let mut handles = vec![];
        let now  = Instant::now();

        for thread_idx in 0..num_threads {
            let data_ptr = SendableU8Pointer(self.data.as_mut_ptr());

            let handle = thread::spawn(move || {
                let _ = &data_ptr;
                let ptr: *mut u8 = data_ptr.0;

                let begin_row = thread_idx * num_row_per_thread;
                let mut end_row: usize = (thread_idx + 1) * num_row_per_thread;
                if thread_idx == (num_threads - 1) {
                    end_row = height; 
                }
                
                for row in begin_row..end_row {
                    let span = width * 3;
                    let mut buf = vec![0 as u8; span];
                    let shift = row * width * 3;
                    
                    unsafe{
                        std::ptr::copy_nonoverlapping(ptr.add(shift), buf.as_mut_ptr(), span);
                    }

                    for col in 0..(width / 2) {
                        let excange_col = width - col;
                        unsafe {
                            exchange_two_pixels(buf.as_mut_ptr(), (0, col as u32, 0, excange_col as u32), width as u32, height as u32, 3);

                        }
                    }
                    unsafe {
                        std::ptr::copy_nonoverlapping(buf.as_ptr(), ptr.add(shift), span);
                    }
                }

            });

            handles.push(handle);
            
        }
   
        for handle in handles {
            handle.join().unwrap();
        }
        
        return now.elapsed().as_micros();
        


    }

    pub fn save(&self, filename: String) {
        let mut res = vec![];
        res.extend_from_slice(&self.header);
        res.extend_from_slice(&self.data);
        res.extend_from_slice(&self.tail);
        fs::write(filename, res).unwrap();
    }
}
