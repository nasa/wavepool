# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-??

### Added

- Telemetry now includes `alpha` (angle-of-attack), `airspeed`, `wind` in 3D, and `distance`

### Changed

- Replaced separate `{Flight,Physics,Weather}Telemetry` structs with a single `Telemetry` struct to simplify analysis later

### Fixed

- `ke` (kinetic energy) and `ma` are now correctly computed from airspeed instead of ground speed

## 0.1.0 - 2026-07-15

[unreleased]: https://github.com/nasa/wavepool/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nasa/wavepool/releases/tag/v0.2.0
