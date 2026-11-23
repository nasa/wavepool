use anyhow::Result;
use std::path::Path;
use wavepool::log::FlightRecorder;
use wavepool::controls::FlightAction;
use wavepool::simulation::{InitialConditions, Sortie};
use wavepool::telemetry::{FlightTelemetry, PhysicsTelemetry, WeatherTelemetry};
use wavepool::time::Benchmark;

fn main() -> Result<()> {
    println!("Hello, world! Let's go wave soaring on Mars!");

    let icons = InitialConditions::new(20000.0, 100.0, -1.7);

    let mut sortie = Sortie::new(Path::new("config/mars.toml"), icons)?; // km, m/s, m/s
    let dt = 0.01; // s
    let mut state = icons.state();

    // benchmark
    let wall_clk = Benchmark::start();
    let t_f: f64 = 88775.0; // s, 1 Sol

    // logging
    let mut black_box = FlightRecorder::new();

    black_box.log_flight_data(FlightTelemetry::from(state));
    black_box.log_physics_data(PhysicsTelemetry::from(state, &sortie.dynamics));
    black_box.log_weather_data(WeatherTelemetry::from(state, &sortie.domain.planet.atmosphere));

    let hard_deck = 10000.0; // m

    while state.altitude() > hard_deck && state.time < t_f {
        // let mut alpha = 10.3_f64.to_radians(); // for 10.6 m/s tailwind
        let mut alpha = 2.0_f64.to_radians(); // for no tailwind
        // alpha = alpha.min(0.44).max(-0.12);

        let action = FlightAction::new(alpha);

        sortie.step_forward(dt, action);
        state = sortie.state;

        // save state
        black_box.log_flight_data(FlightTelemetry::from(state));
        black_box.log_physics_data(PhysicsTelemetry::from(state, &sortie.dynamics));
        black_box.log_weather_data(WeatherTelemetry::from(state, &sortie.domain.planet.atmosphere));
    }

    black_box.shutdown();

    println!("final state: {:#?}", state);
    let range = state.position.x - icons.state().position.x;
    println!(
        "This flight lasted {:.2} minutes and reached {:.2} km from an initial altitude of {:.1} km.",
        state.time / 60.0, // min
        range / 1000.0, // km
        icons.state().altitude() / 1000.0 // km
    );
    println!(
        "The effective glide ratio is {:.1}:1.",
        range / (icons.state().altitude() - state.altitude())
    );

    wall_clk.stop(std::time::Duration::from_secs_f64(state.time));


    Ok(())
}
