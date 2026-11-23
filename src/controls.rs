use crate::dynamics::FlightState;

use nalgebra::{Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct FlightObservation {
    initial_state: FlightState,
    state: FlightState,
    wind: Vector3<f64>
}

impl FlightObservation {
    pub fn from_state(initial_state: FlightState, state: FlightState, wind: Vector3<f64>) -> Self {
         Self {
             initial_state,
             state,
             wind
         }
     }

    // sensed directly
    pub fn altitude(&self) -> f64 {
        // sensor: barometric altimeter
        return self.state.position.z;
    }

    pub fn ground_speed(&self) -> f64 {
        Vector2::new(self.state.velocity.x, self.state.velocity.y).norm()
    }

    pub fn airspeed(&self) -> f64 {
        (self.state.velocity - self.wind).norm()

    }

    // derived
    pub fn gamma(&self) -> f64 {
        self.state.velocity.z.atan2(self.ground_speed())
    }

    pub fn distance(&self) -> f64 {
        (Vector2::new(self.state.position.x, self.state.position.y) - Vector2::new(self.initial_state.position.x, self.initial_state.position.y)).norm()
    }
}


#[derive(Debug, Clone, Copy)]
pub struct FlightAction {
    pub alpha: f64, // rad, angle of attack
}

impl FlightAction {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha
        }
    }
}
