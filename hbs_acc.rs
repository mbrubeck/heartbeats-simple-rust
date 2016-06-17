use heartbeats_simple_sys::*;
use std::mem;
use std::io::{self, Write};
use std::fs::File;

pub use heartbeats_simple_sys::heartbeat_acc_context as HeartbeatAccContext;
pub use heartbeats_simple_sys::heartbeat_acc_record as HeartbeatAccRecord;
pub use heartbeats_simple_sys::heartbeat_acc_window_complete as HeartbeatAccWindowComplete;

/// Contains the Heartbeat and its window data buffer.
pub struct HeartbeatAcc {
    pub hb: HeartbeatAccContext,
    pub hbr: Vec<HeartbeatAccRecord>,
    pub log: Option<File>,
}

impl HeartbeatAcc {
    /// Allocate and initialize a new `Heartbeat`.
    pub fn new(window_size: usize,
               hwc_callback: Option<HeartbeatAccWindowComplete>,
               mut log: Option<File>) -> Result<HeartbeatAcc, &'static str> {
        let mut hbr: Vec<HeartbeatAccRecord> = Vec::with_capacity(window_size);
        let hb: HeartbeatAccContext = unsafe {
            // must explicitly set size so we can read data later
            // (Rust isn't aware of native code modifying the buffer)
            hbr.set_len(window_size);
            let mut hb = mem::uninitialized();
            match heartbeat_acc_init(&mut hb,
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
                         {:15} {:15} {:15} \
                         {:15} {:15} {:15} \
                         {:15} {:15} {}\n",
                        "HB", "Tag",
                        "Global_Work", "Window_Work", "Work",
                        "Global_Time", "Window_Time", "Start_Time", "End_Time",
                        "Global_Perf", "Window_Perf", "Instant_Perf",
                        "Global_Acc", "Window_Acc", "Acc",
                        "Global_Acc_Rate", "Window_Acc_Rate", "Instant_Acc_Rate")
                        .as_bytes()).unwrap()
        }
        Ok(HeartbeatAcc { hb: hb, hbr: hbr, log: log, })
    }

    /// Issue a heartbeat
    pub fn heartbeat(&mut self,
                     tag: u64,
                     work: u64,
                     start_time: u64,
                     end_time: u64,
                     accuracy: u64) {
        unsafe {
            heartbeat_acc(&mut self.hb, tag, work, start_time, end_time, accuracy)
        }
    }

    fn write_log(r: &HeartbeatAccRecord, l: &mut File) -> io::Result<usize> {
        l.write(format!("{:<6} {:<6} \
                         {:<11} {:<11} {:<11} \
                         {:<15} {:<15} {:<20} {:<20} \
                         {:<15.6} {:<15.6} {:<15.6} \
                         {:<15} {:<15} {:<15} \
                         {:<15.6} {:<15.6} {:<.6}\n",
                        r.id, r.user_tag,
                        r.wd.global, r.wd.window, r.work,
                        r.td.global, r.td.window, r.start_time, r.end_time,
                        r.perf.global, r.perf.window, r.perf.instant,
                        r.ad.global, r.ad.window, r.accuracy,
                        r.acc.global, r.acc.window, r.acc.instant).as_bytes())
    }

    /// Rust-only function that logs the buffer (up to buffer_index) to a file.
    pub fn log_to_buffer_index(&mut self) -> io::Result<()> {
        match self.log {
            Some(ref mut l) => {
                for i in 0..self.hb.ws.buffer_index {
                    match HeartbeatAcc::write_log(self.hbr.get(i as usize).unwrap(), l) {
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
            hb_acc_get_window_size(&self.hb)
        }
    }

    pub fn get_user_tag(&self) -> u64 {
        unsafe {
            hb_acc_get_user_tag(&self.hb)
        }
    }

    pub fn get_global_time(&self) -> u64 {
        unsafe {
            hb_acc_get_global_time(&self.hb)
        }
    }

    pub fn get_window_time(&self) -> u64 {
        unsafe {
            hb_acc_get_window_time(&self.hb)
        }
    }

    pub fn get_global_work(&self) -> u64 {
        unsafe {
            hb_acc_get_global_work(&self.hb)
        }
    }

    pub fn get_window_work(&self) -> u64 {
        unsafe {
            hb_acc_get_window_work(&self.hb)
        }
    }

    pub fn get_global_perf(&self) -> f64 {
        unsafe {
            hb_acc_get_global_perf(&self.hb)
        }
    }

    pub fn get_window_perf(&self) -> f64 {
        unsafe {
            hb_acc_get_window_perf(&self.hb)
        }
    }

    pub fn get_instant_perf(&self) -> f64 {
        unsafe {
            hb_acc_get_instant_perf(&self.hb)
        }
    }

    pub fn get_global_accuracy(&self) -> u64 {
        unsafe {
            hb_acc_get_global_accuracy(&self.hb)
        }
    }

    pub fn get_window_accuracy(&self) -> u64 {
        unsafe {
            hb_acc_get_window_accuracy(&self.hb)
        }
    }

    pub fn get_global_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_get_global_accuracy_rate(&self.hb)
        }
    }

    pub fn get_window_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_get_window_accuracy_rate(&self.hb)
        }
    }

    pub fn get_instant_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_get_instant_accuracy_rate(&self.hb)
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
        let mut hb = HeartbeatAcc::new(5, None, None).unwrap();
        let mut start_time: u64 = 0;
        let mut end_time: u64 = TIME_INC;
        for tag in 0..10 {
            hb.heartbeat(tag, 1, start_time, end_time, 1);
            start_time = end_time;
            end_time += TIME_INC;
        }
    }

    #[test]
    fn test_callback() {
        static mut received_cb: bool = false;
        extern fn callback(_hb: *const HeartbeatAccContext) {
            unsafe {
                received_cb = true;
            }
        }

        let mut hb = HeartbeatAcc::new(1, Some(callback), None).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 1);
        unsafe {
            assert!(received_cb);
        }
    }

    #[test]
    fn test_file() {
        let mut hb = HeartbeatAcc::new(5, None, Some(File::create("foo.log").unwrap())).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 1);
        hb.log_to_buffer_index().unwrap();
    }
}
