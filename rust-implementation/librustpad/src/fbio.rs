#![allow(dead_code)]

use fb;

impl<'a> fb::Framebuffer<'a> {
    pub fn write_frame(&mut self, frame: &[u8]) {
        unsafe {
            let begin = self.frame.data() as *mut u8;
            for (i, elem) in frame.iter().enumerate() {
                *(begin.offset(i as isize)) = *elem;
            }
        }
    }

    pub fn write_pixel(&mut self, y: usize, x: usize, v: u8) {
        let w = self.var_screen_info.xres as usize;
        let h = self.var_screen_info.yres as usize;
        if y >= h || x >= w {
            return;
        }
        let line_length = self.fix_screen_info.line_length as usize;
        let bytespp = (self.var_screen_info.bits_per_pixel / 8) as usize;
        let curr_index = (y * line_length + x * bytespp) as isize;

        let begin = self.frame.data() as *mut u8;
        unsafe {
            // TODO: Figure out this packing
            *(begin.offset(curr_index)) = v;
            *(begin.offset(curr_index + 1)) = v;
            *(begin.offset(curr_index + 2)) = v;
        }
    }

    pub fn read_pixel(&mut self, y: usize, x: usize) -> u8 {
        let w = self.var_screen_info.xres as usize;
        let h = self.var_screen_info.yres as usize;
        if y >= h || x >= w {
            return 0;
        }
        let line_length = self.fix_screen_info.line_length as usize;
        let bytespp = (self.var_screen_info.bits_per_pixel / 8) as usize;
        let curr_index = y * line_length + x * bytespp;
        return self.read_offset(curr_index as isize);
    }

    pub fn read_offset(&mut self, ofst: isize) -> u8 {
        unsafe {
            let begin = self.frame.data() as *mut u8;
            return *(begin.offset(ofst));
        }
    }
}
