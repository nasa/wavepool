use crate::telemetry::{Channel, FlightTelemetry, PhysicsTelemetry, WeatherTelemetry};
use chrono::Local;
use std::{
    fs,
    path::Path
};

const CHUNK_SIZE: usize = 100 * 60 * 60 * 10; // at 0.01s of dt, 10 hrs of sim time logged per chunk

pub struct FlightRecorder {
    flight_tlm_channel: Channel<FlightTelemetry>,
    physics_tlm_channel: Channel<PhysicsTelemetry>,
    weather_tlm_channel: Channel<WeatherTelemetry>,
}

impl FlightRecorder {
    pub fn new() -> Self {
        let base_path = format!("tlm/{}", Local::now().format("%Y-%m-%dT%H%M")); // user readable, e.g., 2025-07-08T1307

        fs::create_dir_all(Path::new(&base_path)).unwrap();

        let flt_tlm_tag = format!("{}/flight", base_path);
        let phys_tlm_tag = format!("{}/physics", base_path);
        let weather_tlm_tag = format!("{}/weather", base_path);

        Self {
            flight_tlm_channel: Channel::<FlightTelemetry>::new(flt_tlm_tag, CHUNK_SIZE),
            physics_tlm_channel: Channel::<PhysicsTelemetry>::new(phys_tlm_tag, CHUNK_SIZE),
            weather_tlm_channel: Channel::<WeatherTelemetry>::new(weather_tlm_tag, CHUNK_SIZE),
        }
    }

    pub fn log_flight_data(&mut self, tlm: FlightTelemetry) {
        self.flight_tlm_channel.write(tlm);
    }

    pub fn log_physics_data(&mut self, tlm: PhysicsTelemetry) {
        self.physics_tlm_channel.write(tlm);
    }

    pub fn log_weather_data(&mut self, tlm: WeatherTelemetry) {
        self.weather_tlm_channel.write(tlm);
    }

    pub fn shutdown(&mut self) {
        self.flight_tlm_channel.close();
        self.physics_tlm_channel.close();
        self.weather_tlm_channel.close();
    }
}
