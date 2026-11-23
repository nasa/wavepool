use std::f64::consts::PI;

use crate::time::Timestamp;
use nalgebra::{Vector2, Vector3};
use serde::Deserialize;

const R: f64 = 8.314; // J/(mol*K), universal gas constant

#[derive(Deserialize, Clone, Copy)]
pub struct Atmosphere {
    rho_0: f64,            // kg/m^3, air density at surface
    surface_temp: f64,     // K, average temperature at surface
    air_mass: f64,         // kg/mol
    mu_0: f64,             // kg/m-sa
    s: f64,                // K, Sutherland constant
    reference_temp: f64,   // K, reference temperature
    lapse_rate: f64,       // K/m, lapse rate
    c: f64,                // m/s, speed of sound
    cp: f64,               // J/kg*K
    pbl_day_height: f64,   // avg height of boundary layer during daytime
    pbl_night_height: f64, // avg height of boundary layer during night
    #[serde(skip)]
    pub g: f64, // m/s^2, gravity
}

impl Atmosphere {
    pub fn scale_height(&self) -> f64 {
        (R * self.surface_temp) / (self.air_mass * self.g)
    }

    pub fn rho(&self, altitude: f64) -> f64 {
        // air density
        self.rho_0 * (-altitude / self.scale_height()).exp()
    }

    pub fn q(&self, altitude: f64, air_speed: f64) -> f64 {
        0.5 * self.rho(altitude) * air_speed * air_speed
    }

    pub fn dynamic_viscosity(&self, altitude: f64) -> f64 {
        // mu
        let temperature = self.air_temperature(altitude);
        self.mu_0 * (temperature / self.reference_temp).powf(1.5) * (self.reference_temp + self.s)
            / (temperature + self.s)
    }

    pub fn air_temperature(&self, altitude: f64) -> f64 {
        self.surface_temp + self.lapse_rate * altitude
    }

    // pbl is planetary boundary layer
    pub fn pbl_height(&self, is_daytime: bool) -> f64 {
        if is_daytime {
            self.pbl_day_height
        } else {
            self.pbl_night_height
        }
    }

    pub fn mach(&self, air_speed: f64) -> f64 {
        return air_speed / self.c;
    }
}

#[derive(Deserialize, Clone)]
pub struct Orbit {
    eccentricity: f64,
    mean_motion: f64,
    perihelion_ls: f64,
    pub epoch_ls: f64,
}

impl Orbit {
    pub fn ls_from_sols(&self, elapsed_sols: f64) -> f64 {
        let mean_anomaly = (self.mean_motion * elapsed_sols).to_radians();
        let ecc_anomaly = self.solve_kepler(mean_anomaly, self.eccentricity);

        let true_anomaly = self.eccentric_to_true_anomaly(ecc_anomaly);
        (true_anomaly.to_degrees() + self.perihelion_ls + self.epoch_ls) % 360.0
    }

    pub fn solve_kepler(&self, mean_anomaly: f64, eccentricity: f64) -> f64 {
        let mut ecc_anomaly = mean_anomaly;
        for _ in 1..5 {
            // should converge in just a few iterations
            let delta = ecc_anomaly - eccentricity * ecc_anomaly.sin() - mean_anomaly;

            let delta_p = 1.0 - eccentricity * ecc_anomaly.cos();

            ecc_anomaly -= delta / delta_p;

            if delta.abs() < 1.0e-8 {
                break;
            }
        }
        ecc_anomaly
    }

    pub fn eccentric_to_true_anomaly(&self, ecc_anomaly: f64) -> f64 {
        let n = (1.0 + self.eccentricity).sqrt() * (ecc_anomaly / 2.0).tan();
        let d = (1.0 - self.eccentricity).sqrt();
        2.0 * (n / d).atan()
    }
}

#[derive(Deserialize, Clone)]
pub struct Planet {
    pub g: f64,                // m/s^2, gravitational constant
    pub sol: f64,              // hours in a Sol
    pub obliquity: f64,        // deg, axial tilt
    pub roughness_length: f64, // m
    pub atmosphere: Atmosphere,
    pub orbit: Orbit,
}

impl Planet {
    pub fn solar_declination(&self, ls: f64) -> f64 {
        let ls_rad = ls.to_radians();
        (self.obliquity.to_radians().sin() * ls_rad.sin()).asin()
    }

    pub fn is_daytime(&self, local_time: f64, ls: f64, latitude: f64) -> (bool, f64) {
        // sun declination
        let d = self.solar_declination(ls);

        // local time is [0, sol)
        let hour_angle = (local_time - self.sol / 2.0) * 2.0 * PI / self.sol;

        let lat = latitude.to_radians();

        // sun elevation
        let e = (lat.sin() * d.sin() + lat.cos() * d.cos() * hour_angle.cos()).asin();

        let is_day = e > 0.0;

        (is_day, e)
    }

    pub fn sunrise_sunset(&self, ls: f64, latitude: f64) -> Option<(f64, f64)> {
        let d = self.solar_declination(ls);
        let lat = latitude.to_radians();

        let cos_hour_angle = -(lat.tan() * d.tan());
        if cos_hour_angle > 1.0 {
            return None; // polar night
        } else if cos_hour_angle < -1.0 {
            return Some((0.0, self.sol));
        }

        let hour_angle_hour = cos_hour_angle.acos().to_degrees() * self.sol / 360.0;

        let sunrise = self.sol / 2.0 - hour_angle_hour;
        let sunset = self.sol / 2.0 + hour_angle_hour;

        Some((sunrise, sunset))
    }
}

pub struct TerrainFeature {
    position: Vector2<f64>,
    height: f64,  // m
    width_x: f64, // m FIXME: should be east-west
    width_y: f64, // m FIXME: should be north-south
}

#[derive(Clone)]
pub struct AtmosphericGravityWaveField {
    planet: Planet,
    polar_jet_wind_speed: f64,     // m/s
    polar_jet_wind_direction: f64, // rad
    max_vertical_velocity: f64,
    critical_ri: f64,
    max_l_over_k: f64,
}

impl AtmosphericGravityWaveField {
    pub fn new(planet: Planet, polar_jet_wind_speed: f64, polar_jet_wind_direction: f64) -> Self {
        Self {
            planet,
            polar_jet_wind_speed,
            polar_jet_wind_direction,
            max_vertical_velocity: 5.0,
            critical_ri: 0.25,
            max_l_over_k: 2.0,
        }
    }

    // calculates total wind vector experienced by sailplane
    pub fn wind_vector_at(
        &self,
        position: Vector3<f64>,
        timestamp: Timestamp,
        latitude: f64,
    ) -> Vector3<f64> {
        // FIXME: right now position is decoupled from lat/lon, which we should fix with transforms

        let altitude = position.z;

        let planet = &self.planet;
        let orbit = &planet.orbit;
        let atm = &planet.atmosphere;

        // ls is L_s, 0 for southern fall equinox, 180 for southern vernal equinox
        let ls = orbit.ls_from_sols(timestamp.elapsed_sols);

        // local solar time is [0, 24.6] hours
        let (is_daytime, _) = planet.is_daytime(timestamp.local_solar_time, ls, latitude);

        let pbl_height = atm.pbl_height(is_daytime);

        let (background_wind_speed, _, _) =
            self.wind_at(self.polar_jet_wind_speed, altitude, pbl_height);

        // println!("background_wind_speed: {:.2}", background_wind_speed);

        // background wind experience no terrain, but varies with the planetary boundary layer
        let background_wind = Vector3::new(
            background_wind_speed * self.polar_jet_wind_direction.cos(),
            background_wind_speed * self.polar_jet_wind_direction.sin(),
            0.0,
        );

        let mut all_waves = Vector3::zeros();

        for n in 0..10 {
            let ridge = TerrainFeature {
                position: Vector2::new(0.0 + (n as f64) * 2.5e5, 0.0),
                height: 2000.0,
                width_x: 8000.0,
                width_y: 8000.0,
            };

            all_waves += self.wave_profile_at(position, &ridge, pbl_height);
        }

        //Vector3::zeros() // no wind
        //background_wind // background wind only
        background_wind + all_waves
    }

    // background wind,TODO: should be from EMARS
    fn wind_at(&self, u_ref: f64, altitude: f64, pbl_height: f64) -> (f64, f64, f64) {
        let z0 = self.planet.roughness_length;

        if altitude > z0 && altitude < pbl_height {
            // logarithmic approximation below boundary layer
            let u = u_ref * (altitude / z0).ln() / (pbl_height / z0).ln();
            let du_dz = u_ref / (altitude * (pbl_height / z0).ln());
            let d2u_dz2 = -u_ref / (altitude * altitude * (pbl_height / z0).ln());

            (u, du_dz, d2u_dz2)
        } else if altitude >= pbl_height {
            // linear shear above boundary layer
            let shear_rate = 0.0001; // 10% per km

            let u = u_ref * (1.0 + shear_rate * (altitude - pbl_height));
            let du_dz = u_ref * shear_rate;

            (u, du_dz, 0.0)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /*
     * aka buoyancy frequency, denoted as N
     * atm.lapse_rate (aka environmental lapse rate) is < 0
     * in winter, it is less negative, while in summer it is more negative and almost equal to atm.g/atm.cp
     * N^2 > 0 is stable and support waves, while N^2 < 0 is unstable
     * therefore, flying in winter is advantageous for mountain wave soaring
     */
    fn brunt_vaisala_frequency(&self, altitude: f64) -> f64 {
        let atm = &self.planet.atmosphere;

        let temp = atm.air_temperature(altitude);

        let n_squared = atm.g / temp * (atm.lapse_rate + atm.g / atm.cp);

        if n_squared > 0.0 {
            n_squared.sqrt() // stable stratification
        } else {
            0.0 // neutral or unstable stratification
        }
    }

    /*
     * denoted as L, L^2 = N^2/U^2 - 1/U*d2u/dz2
     * L is a wave number (k = 2*pi/lambda) with units 1/m
     * so, the larger L is, the smaller lambda, and therefore, the more rapidly the wave is oscillating
     * if L^2 > 0, waves are propagating vertically (see "mountain lee waves"), typically L < k
     * if L^2 < 0, waves are exponentially decaying and evanescent (trapped), typically L > k
     */
    fn scorer_parameter(&self, u_ref: f64, altitude: f64, pbl_height: f64) -> f64 {
        let (u, _, d2u_dz2) = self.wind_at(u_ref, altitude, pbl_height);

        if u > 0.0 {
            let n = self.brunt_vaisala_frequency(altitude);

            let l_squared = (n * n) / (u * u) - d2u_dz2 / u;

            if l_squared > 0.0 {
                l_squared.sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    // calculates effective wind speed by integrating over mountain height
    /*
    fn calculate_effective_wind_for(&self, ridge: &TerrainFeature, pbl_height: f64) -> f64 {
        let n_samples = 10;

        let mut total_wind_speed: f64 = 0.0;
        let mut total_w: f64 = 0.0;

        for i in 0..n_samples {
            let height = ridge.height * (i as f64 + 0.5) / n_samples as f64;

            let (wind_speed_at_height, _, _) =
                self.wind_at(self.polar_jet_wind_speed, height, pbl_height);

            let w = 1.0; // TODO: assuming w have a ridge, but can expand to different types of terrain features

            /*
            println!(
                "wind_speed_at: ({:.2}, {:.2})",
                wind_speed_at_height, height
            );
            */

            total_wind_speed += w * wind_speed_at_height;
            total_w += w;
        }

        total_wind_speed / total_w
    }
    */

    fn wave_profile_at(
        &self,
        position: Vector3<f64>,
        ridge: &TerrainFeature,
        pbl_height: f64,
    ) -> Vector3<f64> {
        // 0. Geometry – wind‑aligned coordinates
        let altitude = position.z;

        // Unit vector of the background (polar‑jet) wind
        let wind_dir = Vector3::new(
            self.polar_jet_wind_direction.cos(),
            self.polar_jet_wind_direction.sin(),
            0.0,
        );

        // Horizontal offsets from ridge crest
        let dx = position.x - ridge.position.x;
        let dy = position.y - ridge.position.y;

        // (x′, y′) – coordinates aligned with / across the wind direction
        let x_prime = dx * wind_dir.x + dy * wind_dir.y;
        let y_prime = dx * (-wind_dir.y) + dy * wind_dir.x;

        // Lateral Gaussian envelope (decays away from ridge centre)
        let f_lateral = (-y_prime.powi(2) / (2.0 * ridge.width_y.powi(2))).exp();

        // 1. Background flow & stability parameters
        let (u_eff, du_dz, _) = self.wind_at(self.polar_jet_wind_speed, altitude, pbl_height);

        // Brunt‑Väisälä (buoyancy) frequency at this altitude
        let n = self.brunt_vaisala_frequency(altitude);

        // Scorer parameter L (used for windward‑side scaling)
        let mut l = self.scorer_parameter(u_eff, altitude, pbl_height);
        if !l.is_finite() {
            l = 0.0;
        }

        // Dominant horizontal wavenumber – ridge treated as a single sinusoid
        let k = 2.0 * PI / ridge.width_x;

        // 2. Vertical‑velocity amplitude
        let h_scale = self.planet.atmosphere.scale_height();
        let f_altitude = (-altitude / h_scale).exp();

        // Froude number check, linear theory valid when Fr > ~1.5
        // Fr = U / (N*h)
        let froude = if n > 1e-6 && ridge.height > 1e-6 {
            u_eff / (n * ridge.height)
        } else {
            f64::INFINITY // no wave
        };

        let froude_scale = if froude < 1.0 {
            0.1 // severe flow blocking, waves are supressed
        } else if froude < 1.5 {
            0.1 + 0.9 * (froude - 1.0) / 0.5 // interpolation from linear theory
        } else {
            1.0 // linear theory regime
        };

        // Basic amplitude (proportional to wind × ridge slope)
        let w_amp = u_eff * ridge.height * k * froude_scale;



        // 3. Raw vertical perturbation
        let w_raw = if x_prime < 0.0 {
            // ----- windward (up‑stream) side -----
            if l < k {
                // Propagating vertical structure (rare)
                let m = (k * k - l * l).sqrt();
                let vert = (m * altitude).cos().abs();
                let horiz = (k * x_prime).cos().abs();
                let f_up = (l * x_prime).exp(); // upstream decay
                w_amp * f_altitude * f_up * vert * horiz
            } else {
                // Strongly evanescent upstream tail
                let kappa = (l * l - k * k).sqrt();
                let vert = (-kappa * altitude).exp(); // > 0
                let horiz = (k * x_prime).cos().abs();
                let f_up = (l * x_prime).exp();
                w_amp * f_altitude * f_up * vert * horiz
            }
        } else {
            // ----- lee (down‑stream) side -----
            if l < k {
                // Propagating lee wave
                let m = (k * k - l * l).sqrt();
                let vert = (m * altitude).sin(); // can be ±
                let horiz = (k * x_prime - PI / 2.0).sin(); // ±
                let decay = (-l * x_prime).exp();
                w_amp * f_altitude * decay * vert * horiz
            } else {
                // Evanescent lee wave (trapped)
                let kappa = (l * l - k * k).sqrt();
                let vert = (-kappa * altitude).exp(); // > 0
                let horiz = (k * x_prime - PI / 2.0).sin(); // ±
                let decay = (-kappa * x_prime).exp();
                w_amp * f_altitude * decay * vert * horiz
            }
        };

        // 4. Richardson‑number based saturation
        let ri = if du_dz.abs() > 1e-6 {
            n * n / (du_dz * du_dz)
        } else {
            f64::INFINITY // essentially stable
        };

        let w_limited = if ri < self.critical_ri {
            // Unstable – enforce the ceiling while preserving sign
            let sign = w_raw.signum();
            sign * w_raw.abs().min(self.max_vertical_velocity)
        } else {
            // Stable – keep the raw value (already attenuated by altitude)
            w_raw
        };

        // 5. Horizontal wind perturbation
        let l_over_k = (l / k).min(self.max_l_over_k);
        let n_over_k = (n / (k * u_eff)).min(self.max_l_over_k);

        let (u_pert, w_pert) = if x_prime < 0.0 {
            // Windward side – slow‑to‑zero (negative sign)
            (-l_over_k * w_limited, w_limited)
        } else {
            // Lee side – speed‑up (positive sign)
            (n_over_k * w_limited, w_limited)
        };

        // 6. Rotate the horizontal perturbation back to the global frame
        let u_vec = Vector3::new(u_pert * wind_dir.x, u_pert * wind_dir.y, 0.0);

        // 7. Apply the lateral envelope and return the full perturbation vector
        Vector3::new(u_vec.x * f_lateral, u_vec.y * f_lateral, w_pert * f_lateral)
    }
}
