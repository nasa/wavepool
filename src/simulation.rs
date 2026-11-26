use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use nalgebra::Vector3;

use crate::aircraft::Aircraft;
use crate::atmosphere::{AtmosphericGravityWaveField, Planet};
use crate::controls::{FlightAction, FlightObservation};
use crate::dynamics::{FlightDynamics, FlightState};
use crate::time::Clock;

#[derive(Clone, Copy)]
pub struct InitialConditions {
    altitude: f64,
    v_x: f64,
    v_z: f64,
}

impl InitialConditions {
    pub fn new(altitude: f64, v_x: f64, v_z: f64) -> Self {
        Self {
            altitude,
            v_x,
            v_z
        }
    }

    pub fn state(&self) -> FlightState {
        FlightState {
            time: 0.0,
            position: Vector3::new(0.0, 0.0, self.altitude),
            velocity: Vector3::new(self.v_x, 0.0, self.v_z),
        }
    }
}

#[derive(Deserialize)]
pub struct Domain {
    pub aircraft: Aircraft,
    pub planet: Planet
}

impl Domain {
    fn new(url: &Path) -> Result<Self> {
        let toml_string : String = fs::read_to_string(url)?;
        let mut domain: Self = toml::from_str(toml_string.as_str())?;
        domain.planet.atmosphere.g = domain.planet.g;

        Ok(domain)
    }
}

pub struct Sortie {
    pub clk: Clock,
    pub domain: Domain,
    pub dynamics: FlightDynamics,
    initial_state: FlightState,
    pub state: FlightState,
    wind: AtmosphericGravityWaveField
}

impl Sortie {
    pub fn new(url: &Path, icons: InitialConditions) -> Result<Self> {
        let domain = Domain::new(url)?;

        let clk = Clock::new(domain.planet.sol);
        let dynamics = FlightDynamics::new(&domain.aircraft, &domain.planet);
        let wind = AtmosphericGravityWaveField::new(domain.planet.clone(), 10.6, 0.0);

        let initial_state = icons.state();

        Ok(Self {
            clk,
            domain,
            dynamics,
            initial_state,
            state: initial_state.clone(),
            wind
        })
    }


    pub fn step_forward(&mut self, dt: f64, action: FlightAction) {
        let v_wind = self.wind.wind_vector_at(self.state.position, self.clk.now(), -60.0);

        let mut alpha = action.alpha;
        alpha = alpha.min(0.44).max(-0.12);

        self.state = self.dynamics.step_forward(self.state, v_wind, alpha, dt);
        self.clk.increment(dt);
    }

    pub fn wind(&self) -> Vector3<f64> {
        self.wind.wind_vector_at(self.state.position, self.clk.now(), -60.0)
    }

    pub fn observe(&self) -> FlightObservation {
       FlightObservation::from_state(self.initial_state, self.state, self.wind())
    }
}
