<p align="center"><img src="docs/img/wavepool_logo.png" height="200" alt="wavepool Logo"></p>

# wavepool

This is a wave soaring sailplane simulator for planetary bodies with wind and mountains. Read these [concepts](docs/concepts.md) for context.

## Key Features

- Mars time, gravity, and atmosphere
- First-order, 2-dimensional flight physics (altitude, attitude)
- First-order atmospheric gravity wave generator (aka. mountain lee waves)

## Limitations
- Static angle-of-attack
- Background wind direction and flight are aligned, mountain ridges are perpendicular
- 2-dimensional dynamics
- Linear atmospheric gravity wave models that work for small background winds, nominal ridge heights
- No wave breaking turbulence
- Constant scale height, which does not account for temperature-dependent density variations
- No turbulence modeling (e.g., wave breaking, lee rotors, etc.), assumes laminar flow fields
- Constant lift curve slope, no accounting for Mach (assuming M < 0.6) or Reynolds number effects on aerodynamics

## Build Instructions

Building and running is handled via `docker`, or can be achieved directly via `cargo` (Rust) or `nix` (Nix).

Build it: `docker build -t wavepool:dev .` (or call it something other than `wavepool:dev` if you like)

Run it: `docker run -u $(id -u):$(id -g) -v "$(pwd)/tlm:/app/tlm" -t wavepool:dev` (make sure `wavepool:dev` matches your name and tag of the image; also, this make sure that telemetry is written to the `tlm/` directory locally with your user's permissions)

An example configuration file is located in `config/`, and the output telemetry is written to a single unified parquet file in `tlm/YYYY-MM-DDTHHMM` (for example, `tlm/2025-08-19T1028/`) in [Parquet](https://parquet.apache.org) format.

## Configuration

Inspect [mars.toml](config/mars.toml) for an example of a configuration for Mars. The configuration file is broken down into the aircraft and the planet. The planet has an atmosphere, and later will have a wind system.

## Development Roadmap

### Controls & Guidance
- [ ] Dynamic angle-of-attack control system (currently static input)
- [ ] Autonomous soaring controller (wave detection, energy optimization, trajectory planning)
- [ ] Waypoint navigation system (follow pre-planned routes)
- [ ] Path planning algorithms (find optimal routes through wave fields)
- [ ] Control surface modeling (elevator, aileron, rudder deflections as they impact forces/moments)
- [ ] Control authority limits (max deflection rates, control saturation)

### Flight Dynamics
- [ ] Full 6-DOF dynamics (roll, pitch, yaw rates)
- [ ] Bank angle and coordinated turns (slip/skid modeling)
- [ ] Stall aerodynamics (post-stall lift/drag, spin dynamics)
- [ ] Enforce stall speed (minimum airspeed for controlled flight)
- [ ] Never-exceed speed (structural limits)
- [ ] Coordinate transformations
- [ ] Latitude-dependent Coriolis effect
- [ ] Great-circle navigation for long-distance flight planning
- [ ] Mars curvature effects (for flights > 1000 km)
- [ ] Spherical coordinate system (proper lat/lon tracking as position changes)

### Aerodynamics
- [ ] Mach-dependent aerodynamics (compressibility effects at M > 0.5)
- [ ] Reynolds-dependent aerodynamics (boundary layer transition, drag variations)
- [ ] Variable stall characteristics (stall varies with Mach, Re, load factor)
- [ ] Wing loading effects (span-wise lift distribution, induced drag refinement)
- [ ] Ground effect modeling (for launch/landing phases)

### Atmospheric Modeling
- [ ] Turbulence modeling (stochastic velocity perturbations from wave breaking, lee rotors)
- [ ] Non-linear wave dynamics (hydraulic jumps, flow separation, wave-wave interactions)
- [ ] Thermal updrafts (convective, not just orographic waves)
- [ ] Time-varying atmospheric state (diurnal wind cycles, boundary layer evolution)
- [ ] Seasonal atmospheric variations (dust loading, temp profiles beyond simple L_s)
- [ ] Integration with Mars atmospheric databases (EMARS, MCD for validated profiles)
- [ ] Dust storm effects (visibility, heating/cooling, turbulence)
- [ ] Atmospheric forecast uncertainty (wind prediction errors)

### Terrain & Environment
- [ ] Realistic Mars terrain from elevation data (MOLA, HiRISE)
- [ ] Arbitrary terrain feature orientations (ridges not perpendicular to wind)
- [ ] Multiple interacting terrain features (wave interference patterns)
- [ ] Surface property variations (roughness, albedo affecting surface heating/thermals)
- [ ] Terrain collision detection and avoidance
- [ ] Operational altitude ceiling (atmosphere too thin above ~50 km)

### Performance & Analysis
- [ ] Energy accounting (KE + PE balance, dissipation tracking, wave energy extraction)
- [ ] Performance metrics (L/D ratio instantaneous & average, sink rate, glide ratio)
- [ ] Soaring efficiency metrics (altitude gain rate, distance covered, energy efficiency)
- [ ] Energy state constraints (can't violate energy conservation)
- [ ] Validation test suite (compare to analytical solutions, Earth glider data scaled to Mars)
- [ ] Trajectory visualization tools (3D path plots, energy-height diagrams)

### Mission Operations
- [ ] Launch phase modeling (release during EDL, i.e., EDF)
- [ ] Landing approach and touchdown (glide slope, flare, ground roll)
- [ ] Communication link simulation (downlink telemetry bandwidth, latency, windows)

### Numerical Methods
- [ ] Adaptive time-stepping (smaller dt in high-gradient regions like wave breaking)
- [ ] Higher-order integration (RK45 with error control for stiff problems)
- [ ] Numerical stability analysis (ensure no spurious oscillations)
- [ ] Performance profiling and optimization (faster-than-realtime simulation target)
- [ ] Parallel simulation capability (Monte Carlo uncertainty analysis)

### Uncertainty & Estimation
- [ ] Sensor noise modeling (GPS, IMU, airspeed, altimeter errors)
- [ ] State estimation (Kalman filter for noisy measurements)
- [ ] Navigation errors and corrections
- [ ] Monte Carlo analysis tools (sensitivity to parameters, atmospheric uncertainty)
