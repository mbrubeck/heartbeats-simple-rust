use heartbeats_simple_sys::pow::{heartbeat_pow_init, heartbeat_pow};
use std::mem;
use std::io::{self, Write};
use std::fs::File;

pub use heartbeats_simple_sys::pow::heartbeat_pow_context as HeartbeatPowContext;
pub use heartbeats_simple_sys::pow::heartbeat_pow_record as HeartbeatPowRecord;
pub use heartbeats_simple_sys::pow::heartbeat_pow_window_complete as HeartbeatPowWindowComplete;

/// Contains the Heartbeat and its window data buffer.
pub struct HeartbeatPow {
    pub hb: HeartbeatPowContext,
    pub hbr: Vec<HeartbeatPowRecord>,
    pub log: Option<File>,
}

impl HeartbeatPow {
    /// Allocate and initialize a new `Heartbeat`.
    pub fn new(window_size: usize,
               hwc_callback: Option<HeartbeatPowWindowComplete>,
               mut log: Option<File>) -> Result<HeartbeatPow, &'static str> {
        let mut hbr: Vec<HeartbeatPowRecord> = Vec::with_capacity(window_size);
        let hb: HeartbeatPowContext = unsafe {
            // must explicitly set size so we can read data later
            // (Rust isn't aware of native code modifying the buffer)
            hbr.set_len(window_size);
            let mut hb = mem::uninitialized();
            match heartbeat_pow_init(&mut hb,
                                     hbr.capacity() as u64,
                                     hbr.as_mut_ptr(),
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
                         {:15} {:15} {:15} {:15} \
                         {:15} {:15} {}\n",
                        "HB", "Tag",
                        "Global_Work", "Window_Work", "Work",
                        "Global_Time", "Window_Time", "Start_Time", "End_Time",
                        "Global_Perf", "Window_Perf", "Instant_Perf",
                        "Global_Energy", "Window_Energy", "Start_Energy", "End_Energy",
                        "Global_Pwr", "Window_Pwr", "Instant_Pwr").as_bytes()).unwrap()
        }
        Ok(HeartbeatPow { hb: hb, hbr: hbr, log: log, })
    }

    /// Issue a heartbeat
    pub fn heartbeat(&mut self,
                     tag: u64,
                     work: u64,
                     start_time: u64,
                     end_time: u64,
                     start_energy: u64,
                     end_energy: u64) {
        unsafe {
            heartbeat_pow(&mut self.hb, tag, work, start_time, end_time, start_energy, end_energy)
        }
    }

    fn write_log(r: &HeartbeatPowRecord, l: &mut File) -> io::Result<usize> {
        l.write(format!("{:<6} {:<6} \
                         {:<11} {:<11} {:<11} \
                         {:<15} {:<15} {:<20} {:<20} \
                         {:<15.6} {:<15.6} {:<15.6} \
                         {:<15} {:<15} {:<15} {:<15} \
                         {:<15.6} {:<15.6} {:<.6}\n",
                        r.id, r.user_tag,
                        r.wd.global, r.wd.window, r.work,
                        r.td.global, r.td.window, r.start_time, r.end_time,
                        r.perf.global, r.perf.window, r.perf.instant,
                        r.ed.global, r.ed.window, r.start_energy, r.end_energy,
                        r.pwr.global, r.pwr.window, r.pwr.instant).as_bytes())
    }

    /// Rust-only function that logs the buffer (up to buffer_index) to a file.
    pub fn log_to_buffer_index(&mut self) -> io::Result<()> {
        match self.log {
            Some(ref mut l) => {
                for i in 0..self.hb.ws.buffer_index {
                    match HeartbeatPow::write_log(self.hbr.get(i as usize).unwrap(), l) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                l.flush()
            }
            None => Ok(())
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
        let mut hb = HeartbeatPow::new(5, None, None).unwrap();
        let mut start_time: u64 = 0;
        let mut end_time: u64 = TIME_INC;
        let mut start_energy: u64 = 0;
        let mut end_energy: u64 = ENERGY_INC;
        for tag in 0..10 {
            hb.heartbeat(tag, 1, start_time, end_time, start_energy, end_energy);
            start_time = end_time;
            end_time += TIME_INC;
            start_energy = end_energy;
            end_energy += ENERGY_INC;
        }
    }

    #[test]
    fn test_callback() {
        static mut received_cb: bool = false;
        extern fn callback(_hb: *const HeartbeatPowContext) {
            unsafe {
                received_cb = true;
            }
        }

        let mut hb = HeartbeatPow::new(1, Some(callback), None).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 0, 0);
        unsafe {
            assert!(received_cb);
        }
    }

    #[test]
    fn test_file() {
        let mut hb = HeartbeatPow::new(5, None, Some(File::create("foo.log").unwrap())).unwrap();
        hb.heartbeat(0, 1, 0, 1000, 0, 0);
        hb.log_to_buffer_index().unwrap();
    }
}
