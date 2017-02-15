use heartbeats_simple_sys::*;
use std::mem;
use std::io::{self, Write};
use std::fs::File;

pub use heartbeats_simple_sys::heartbeat_context as HeartbeatContext;
pub use heartbeats_simple_sys::heartbeat_record as HeartbeatRecord;
pub use heartbeats_simple_sys::heartbeat_window_complete as HeartbeatWindowComplete;

/// Contains the Heartbeat and its window data buffer.
pub struct Heartbeat {
    pub hb: HeartbeatContext,
    pub hbr: Vec<HeartbeatRecord>,
    pub log: Option<File>,
}

impl Heartbeat {
    /// Allocate and initialize a new `Heartbeat`.
    pub fn new(window_size: usize,
               hwc_callback: HeartbeatWindowComplete,
               mut log: Option<File>) -> Result<Heartbeat, &'static str> {
        let mut hbr: Vec<HeartbeatRecord> = Vec::with_capacity(window_size);
        let hb: HeartbeatContext = unsafe {
            // must explicitly set size so we can read data later
            // (Rust isn't aware of native code modifying the buffer)
            hbr.set_len(window_size);
            let mut hb = mem::uninitialized();
            match heartbeat_init(&mut hb,
                                 hbr.capacity() as u64,
                                 hbr.as_mut_ptr(),
                                 -1,
                                 hwc_callback) {
                0 => hb,
                _ => return Err("Failed to initialize heartbeat")
            }
        };
        // write header to log file if there is one
        if let Some(ref mut l) = log {
            l.write_all(format!("{:6} {:6} \
                         {:11} {:11} {:11} \
                         {:15} {:15} {:20} {:20} \
                         {:15} {:15} {}\n",
                        "HB", "Tag",
                        "Global_Work", "Window_Work", "Work",
                        "Global_Time", "Window_Time", "Start_Time", "End_Time",
                        "Global_Perf", "Window_Perf", "Instant_Perf").as_bytes()).unwrap()
        }
        Ok(Heartbeat { hb: hb, hbr: hbr, log: log, })
    }

    /// Issue a heartbeat
    pub fn heartbeat(&mut self,
                     tag: u64,
                     work: u64,
                     start_time: u64,
                     end_time: u64) {
        unsafe {
            heartbeat(&mut self.hb, tag, work, start_time, end_time)
        }
    }

    fn write_log(r: &HeartbeatRecord, l: &mut File) -> io::Result<usize> {
        l.write(format!("{:<6} {:<6} \
                         {:<11} {:<11} {:<11} \
                         {:<15} {:<15} {:<20} {:<20} \
                         {:<15.6} {:<15.6} {:<.6}\n",
                        r.id, r.user_tag,
                        r.wd.global, r.wd.window, r.work,
                        r.td.global, r.td.window, r.start_time, r.end_time,
                        r.perf.global, r.perf.window, r.perf.instant).as_bytes())
    }

    /// Rust-only function that logs the buffer (up to buffer_index) to a file.
    pub fn log_to_buffer_index(&mut self) -> io::Result<()> {
        match self.log {
            Some(ref mut l) => {
                for i in 0..self.hb.ws.buffer_index {
                    match Heartbeat::write_log(self.hbr.get(i as usize).unwrap(), l) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                l.flush()
            }
            None => Ok(())
        }
    }

    pub fn get_window_size(&self) -> u64 {
        unsafe {
            hb_get_window_size(&self.hb)
        }
    }

    pub fn get_user_tag(&self) -> u64 {
        unsafe {
            hb_get_user_tag(&self.hb)
        }
    }

    pub fn get_global_time(&self) -> u64 {
        unsafe {
            hb_get_global_time(&self.hb)
        }
    }

    pub fn get_window_time(&self) -> u64 {
        unsafe {
            hb_get_window_time(&self.hb)
        }
    }

    pub fn get_global_work(&self) -> u64 {
        unsafe {
            hb_get_global_work(&self.hb)
        }
    }

    pub fn get_window_work(&self) -> u64 {
        unsafe {
            hb_get_window_work(&self.hb)
        }
    }

    pub fn get_global_perf(&self) -> f64 {
        unsafe {
            hb_get_global_perf(&self.hb)
        }
    }

    pub fn get_window_perf(&self) -> f64 {
        unsafe {
            hb_get_window_perf(&self.hb)
        }
    }

    pub fn get_instant_perf(&self) -> f64 {
        unsafe {
            hb_get_instant_perf(&self.hb)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_simple() {
        const TIME_INC: u64 = 1000000000;
        let mut hb = Heartbeat::new(5, None, None).unwrap();
        let mut start_time: u64 = 0;
        let mut end_time: u64 = TIME_INC;
        for tag in 0..10 {
            hb.heartbeat(tag, 1, start_time, end_time);
            start_time = end_time;
            end_time += TIME_INC;
        }
    }

    #[test]
    fn test_callback() {
        static mut RECEIVED_CB: bool = false;
        extern fn callback(_hb: *const HeartbeatContext) {
            unsafe {
                RECEIVED_CB = true;
            }
        }

        let mut hb = Heartbeat::new(1, Some(callback), None).unwrap();
        hb.heartbeat(0, 1, 0, 1000);
        unsafe {
            assert!(RECEIVED_CB);
        }
    }

    #[test]
    fn test_file() {
        let mut hb = Heartbeat::new(5, None, Some(File::create("foo.log").unwrap())).unwrap();
        hb.heartbeat(0, 1, 0, 1000);
        hb.log_to_buffer_index().unwrap();
    }
}
