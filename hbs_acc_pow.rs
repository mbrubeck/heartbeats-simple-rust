use heartbeats_simple_sys::*;
use std::mem;
use std::io::{self, Write};
use std::fs::File;

pub use heartbeats_simple_sys::heartbeat_acc_pow_context as HeartbeatAccPowContext;
pub use heartbeats_simple_sys::heartbeat_acc_pow_record as HeartbeatAccPowRecord;
pub use heartbeats_simple_sys::heartbeat_acc_pow_window_complete as HeartbeatAccPowWindowComplete;

/// Contains the Heartbeat and its window data buffer.
pub struct HeartbeatAccPow {
    pub hb: HeartbeatAccPowContext,
    pub hbr: Vec<HeartbeatAccPowRecord>,
    pub log: Option<File>,
}

impl HeartbeatAccPow {
    /// Allocate and initialize a new `Heartbeat`.
    pub fn new(window_size: usize,
               hwc_callback: HeartbeatAccPowWindowComplete,
               mut log: Option<File>) -> Result<HeartbeatAccPow, &'static str> {
        let mut hbr: Vec<HeartbeatAccPowRecord> = Vec::with_capacity(window_size);
        let hb: HeartbeatAccPowContext = unsafe {
            // must explicitly set size so we can read data later
            // (Rust isn't aware of native code modifying the buffer)
            hbr.set_len(window_size);
            let mut hb = mem::uninitialized();
            match heartbeat_acc_pow_init(&mut hb,
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
        Ok(HeartbeatAccPow { hb: hb, hbr: hbr, log: log, })
    }

    /// Issue a heartbeat
    pub fn heartbeat(&mut self,
                     tag: u64,
                     work: u64,
                     start_time: u64,
                     end_time: u64,
                     accuracy: u64,
                     start_energy: u64,
                     end_energy: u64) {
        unsafe {
            heartbeat_acc_pow(&mut self.hb,
                              tag,
                              work,
                              start_time,
                              end_time,
                              accuracy,
                              start_energy,
                              end_energy)
        }
    }

    fn write_log(r: &HeartbeatAccPowRecord, l: &mut File) -> io::Result<usize> {
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
                    match HeartbeatAccPow::write_log(self.hbr.get(i as usize).unwrap(), l) {
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
            hb_acc_pow_get_window_size(&self.hb)
        }
    }

    pub fn get_user_tag(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_user_tag(&self.hb)
        }
    }

    pub fn get_global_time(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_global_time(&self.hb)
        }
    }

    pub fn get_window_time(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_window_time(&self.hb)
        }
    }

    pub fn get_global_work(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_global_work(&self.hb)
        }
    }

    pub fn get_window_work(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_window_work(&self.hb)
        }
    }

    pub fn get_global_perf(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_global_perf(&self.hb)
        }
    }

    pub fn get_window_perf(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_window_perf(&self.hb)
        }
    }

    pub fn get_instant_perf(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_instant_perf(&self.hb)
        }
    }

    pub fn get_global_accuracy(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_global_accuracy(&self.hb)
        }
    }

    pub fn get_window_accuracy(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_window_accuracy(&self.hb)
        }
    }

    pub fn get_global_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_global_accuracy_rate(&self.hb)
        }
    }

    pub fn get_window_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_window_accuracy_rate(&self.hb)
        }
    }

    pub fn get_instant_accuracy_rate(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_instant_accuracy_rate(&self.hb)
        }
    }

    pub fn get_global_energy(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_global_energy(&self.hb)
        }
    }

    pub fn get_window_energy(&self) -> u64 {
        unsafe {
            hb_acc_pow_get_window_energy(&self.hb)
        }
    }

    pub fn get_global_power(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_global_power(&self.hb)
        }
    }

    pub fn get_window_power(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_window_power(&self.hb)
        }
    }

    pub fn get_instant_power(&self) -> f64 {
        unsafe {
            hb_acc_pow_get_instant_power(&self.hb)
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
        const ENERGY_INC: u64 = 1000000;
        let mut hb = HeartbeatAccPow::new(5, None, None).unwrap();
        let mut start_time: u64 = 0;
        let mut end_time: u64 = TIME_INC;
        let mut start_energy: u64 = 0;
        let mut end_energy: u64 = ENERGY_INC;
        for tag in 0..10 {
            hb.heartbeat(tag, 1, start_time, end_time, 1, start_energy, end_energy);
            start_time = end_time;
            end_time += TIME_INC;
            start_energy = end_energy;
            end_energy += ENERGY_INC;
        }
    }

    #[test]
    fn test_callback() {
        static mut RECEIVED_CB: bool = false;
        extern fn callback(_hb: *const HeartbeatAccPowContext) {
            unsafe {
                RECEIVED_CB = true;
            }
        }

        let mut hb = HeartbeatAccPow::new(1, Some(callback), None).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 1, 0, 0);
        unsafe {
            assert!(RECEIVED_CB);
        }
    }

    #[test]
    fn test_file() {
        let mut hb = HeartbeatAccPow::new(5, None, Some(File::create("foo.log").unwrap())).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 1, 0, 0);
        hb.log_to_buffer_index().unwrap();
    }
}
