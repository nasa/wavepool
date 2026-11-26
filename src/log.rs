use crate::telemetry::{Channel, Telemetry};
use chrono::Local;
use std::{
    fs,
    path::Path
};

const CHUNK_SIZE: usize = 100 * 60 * 60 * 10; // at 0.01s of dt, 10 hrs of sim time logged per chunk

pub struct FlightRecorder {
    tlm_channel: Channel<Telemetry>,
}

impl FlightRecorder {
    pub fn new() -> Self {
        let base_path = format!("tlm/{}", Local::now().format("%Y-%m-%dT%H%M")); // user readable, e.g., 2025-07-08T1307

        fs::create_dir_all(Path::new(&base_path)).unwrap();

        let tlm_tag = format!("{}/tlm", base_path);

        Self {
            tlm_channel: Channel::<Telemetry>::new(tlm_tag, CHUNK_SIZE),
        }
    }

    pub fn log(&mut self, tlm: Telemetry) {
        self.tlm_channel.write(tlm);
    }

    pub fn shutdown(&mut self) {
        self.tlm_channel.close();
    }
}
