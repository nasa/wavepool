use std::sync::atomic::AtomicUsize;

use crate::atmosphere::Atmosphere;
use crate::dynamics::{FlightDynamics, FlightState};
use polars::prelude::*;

pub trait Frameable {
    fn to_dataframe(data: &[Self]) -> DataFrame
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Telemetry {
    time: f64,
    gamma: f64,
    x: f64,
    y: f64,
    z: f64,
    v_x: f64,
    v_y: f64,
    v_z: f64,
    pe: f64, // potential energy, PE
    ke: f64, // kinetic energy, KE
    ma: f64, // mach number, Ma
    air_temp: f64,
}

impl Telemetry {
    pub fn from(state: FlightState, dynamics: &FlightDynamics, atmosphere: &Atmosphere) -> Self {
        Self {
            time: state.time,
            gamma: state.gamma(),
            x: state.position.x,
            y: state.position.y,
            z: state.position.z,
            v_x: state.velocity.x,
            v_y: state.velocity.y,
            v_z: state.velocity.z,
            pe: dynamics.potential_energy(state.altitude()),
            ke: dynamics.kinetic_energy(state.velocity.magnitude()),
            ma: dynamics.mach(state.velocity.magnitude()),
            air_temp: atmosphere.air_temperature(state.altitude()),
        }
    }
}

impl Frameable for Telemetry {
    fn to_dataframe(data: &[Self]) -> DataFrame {
        let time: Vec<f64> = data.iter().map(|d| d.time).collect();
        let gamma: Vec<f64> = data.iter().map(|d| d.gamma).collect();
        let x: Vec<f64> = data.iter().map(|d| d.x).collect();
        let y: Vec<f64> = data.iter().map(|d| d.y).collect();
        let z: Vec<f64> = data.iter().map(|d| d.z).collect();
        let v_x: Vec<f64> = data.iter().map(|d| d.v_x).collect();
        let v_y: Vec<f64> = data.iter().map(|d| d.v_y).collect();
        let v_z: Vec<f64> = data.iter().map(|d| d.v_z).collect();
        let pe: Vec<f64> = data.iter().map(|d| d.pe).collect();
        let ke: Vec<f64> = data.iter().map(|d| d.ke).collect();
        let ma: Vec<f64> = data.iter().map(|d| d.ma).collect();
        let air_temp: Vec<f64> = data.iter().map(|d| d.air_temp).collect();

        df! {
            "time" => time,
            "gamma" => gamma,
            "x" => x,
            "y" => y,
            "z" => z,
            "v_x" => v_x,
            "v_y" => v_y,
            "v_z" => v_z,
            "pe" => pe,
            "ke" => ke,
            "ma" => ma,
            "air_temp" => air_temp,
        }
        .unwrap()
    }
}

// channel

pub struct Channel<T> {
    tlm_queue: Vec<T>,
    batch_size: usize,
    batch_index: AtomicUsize,
    base_url: String,
}

impl<T> Channel<T> {
    pub fn new(base_url: String, batch_size: usize) -> Self {
        Self {
            tlm_queue: Vec::with_capacity(batch_size),
            batch_size,
            batch_index: AtomicUsize::new(0),
            base_url,
        }
    }

    pub fn write(&mut self, tlm: T)
    where
        T: Frameable,
    {
        self.tlm_queue.push(tlm);

        if self.tlm_queue.len() >= self.batch_size {
            self.dispatch();
        }
    }

    fn dispatch(&mut self)
    where
        T: Frameable,
    {
        if self.tlm_queue.is_empty() {
            return;
        }

        let df = T::to_dataframe(&self.tlm_queue);

        let batch_num = self
            .batch_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let url = format!("{}_{:06}.parquet", self.base_url, batch_num);

        let mut file = std::fs::File::create(&url).unwrap();
        ParquetWriter::new(&mut file)
            .finish(&mut df.clone())
            .unwrap();

        self.tlm_queue.clear();
    }

    pub fn close(&mut self)
    where
        T: Frameable,
    {
        self.dispatch();
    }
}
