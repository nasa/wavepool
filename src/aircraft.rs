use crate::atmosphere::Atmosphere;
use serde::Deserialize;
use std::f64::consts::PI;

#[derive(Deserialize, Clone, Copy)]
struct Limits {
    max_load_factor: f64, // scale factor of g
    min_load_factor: f64, // scale factor of g
}

#[derive(Deserialize, Clone, Copy)]
struct Aerodynamics {
    cl_0: f64,              // lift coefficient at zero angle of attack
    cl_alpha: f64,          // lift curve slope (per radian)
    cl_min: f64,            // minimum lift coefficient (negative limit)
    cl_max: f64,            // maximum lift coefficient (stall limit)
    cd_0: f64,              // parasitic drag coefficient
    oswald_efficiency: f64, // Oswald efficiency factor, e
}

#[derive(Deserialize, Clone, Copy)]
pub struct Aircraft {
    pub mass: f64,  // kg, m
    wing_area: f64, // m^2, S
    wing_span: f64, // m, b
    limits: Limits,
    aerodynamics: Aerodynamics,
}

impl Aircraft {
    pub fn limit_load_factor(&self, load_factor: f64) -> f64 {
        load_factor
            .min(self.limits.max_load_factor)
            .max(self.limits.min_load_factor)
    }

    pub fn aspect_ratio(&self) -> f64 {
        // AR = b^2/S
        (self.wing_span * self.wing_span) / self.wing_area
    }

    pub fn induced_drag_factor(&self) -> f64 {
        // k = 1/(pi*AR*e)
        1.0 / (PI * self.aspect_ratio() * self.aerodynamics.oswald_efficiency)
    }

    pub fn lift_coefficient(&self, alpha: f64) -> f64 {
        // C_L
        let cl = self.aerodynamics.cl_0 + self.aerodynamics.cl_alpha * alpha;
        cl.min(self.aerodynamics.cl_max)
            .max(self.aerodynamics.cl_min)
    }

    pub fn drag_coefficient(&self, cl: f64) -> f64 {
        // C_D
        self.aerodynamics.cd_0 + self.induced_drag_factor() * cl * cl
    }

    pub fn wing_loading(&self, g: f64) -> f64 {
        // N/m^2, divide by gravity to get kg/m^2
        (self.mass * g) / self.wing_area
    }

    pub fn lift_and_drag(
        &self,
        altitude: f64,
        airspeed: f64,
        atmosphere: Atmosphere,
        input: f64,
    ) -> (f64, f64) {
        let cl = self.lift_coefficient(input);
        let cd = self.drag_coefficient(cl);
        let q = atmosphere.q(altitude, airspeed);

        let lift = q * self.wing_area * cl; // L
        let drag = q * self.wing_area * cd; // D

        (lift, drag)
    }

    pub fn reynolds_number(&self, altitude: f64, airspeed: f64, atmosphere: Atmosphere) -> f64 {
        let chord = self.wing_area / self.wing_span;
        (atmosphere.rho(altitude) * airspeed * chord) / atmosphere.dynamic_viscosity(altitude)
    }
}

impl Aircraft {}
