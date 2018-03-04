use libc::ioctl;

use std;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::Ordering;

use fb;
use mxc_types;
use mxc_types::{mxcfb_update_marker_data, mxcfb_update_data, mxcfb_rect};

impl<'a> fb::Framebuffer<'a> {
    pub fn refresh(
        &mut self,
        y: usize,
        x: usize,
        height: usize,
        width: usize,
        update_mode: mxc_types::update_mode,
        waveform_mode: mxc_types::waveform_mode,
        temperature: mxc_types::display_temp,
        dither_mode: mxc_types::dither_mode,
        quant_bit: i32,
        flags: u32,
    ) -> u32 {
        const SEND_UPDATE_IOCTL: u32 = iow!(b'F', 0x2E, std::mem::size_of::<mxcfb_update_data>());
        let whole = mxcfb_update_data {
            update_mode: update_mode as u32,
            update_marker: *self.marker.get_mut() as u32,
            waveform_mode: waveform_mode as u32,
            temp: temperature as i32,
            flags: flags,
            quant_bit: quant_bit,
            dither_mode: dither_mode as i32,
            update_region: mxcfb_rect {
                top: y as u32,
                left: x as u32,
                height: height as u32,
                width: width as u32,
            },
            ..Default::default()
        };
        let pt: *const mxcfb_update_data = &whole;
        unsafe {
            ::libc::ioctl(self.device.as_raw_fd(), SEND_UPDATE_IOCTL, pt);
        }
        // TODO: Do proper compare and swap
        self.marker.swap(whole.update_marker + 1, Ordering::Relaxed);
        return whole.update_marker;
    }

    pub fn wait_refresh_complete(&mut self, marker: u32) {
        const MXCFB_WAIT_FOR_UPDATE_COMPLETE: u32 =
            iowr!(b'F', 0x2F, std::mem::size_of::<mxcfb_update_marker_data>());
        let mut markerdata = mxcfb_update_marker_data {
            update_marker: marker,
            collision_test: 0,
        };
        unsafe {
            ioctl(
                self.device.as_raw_fd(),
                MXCFB_WAIT_FOR_UPDATE_COMPLETE,
                &mut markerdata,
            );
        };
        // TODO: Return collision test -- kernel updates it to the next marker's collision data
    }
}