# Fractal Synaesthesia

> A high-performance Rust application that transformts music into real-time Mandelbrot fractal visualisations through audio feature extraction and configurable visual mappings.

![Rust](https://img.shields.io/badge/Rust-Programming%20Language-orange)
![FFmpeg](https://img.shields.io/badge/FFmpeg-Video%20Export-green)

<p align="center">
    <img src="./assets/speedy-compressed.gif" alt="Fractal Synaesthesia Demo" width="900">
</p>

## Overview

Fractal Synaesthesia is my final-year Computer Science dissertation project at the University of Warwick.

The application analyses music to extract features such as tempo, beat locations, pitch, loudness and spectral information, then maps those features onto parameters of a Mandelbrot renderer. The result is an interactive visualisation where the fractal evolves in response to the music in real time.

Rather than generating fixed animations, users can create their own mappings between musical characteristics and visual behaviour, allowing every song to produce a unique experience.

---

## Features

- 🎵 Audio feature extraction
    - Beat detection
    - Tempo estimation
    - Pitch tracking
    - Volume analysis
    - Spectral centroid
    - Spectral flux
    - Frequency band energy
    - Emotion (arousal/valence) estimation

- 🌀 High-performance Mandelbrot rendering
    - Deep zoom support
    - Perturbation rendering
    - Arbitrary precision arithmetic
    - Dynamic quality scaling for smooth previews

- 🎨 Configurable visual mappings
    - Beat-triggered events
    - Continuous parameter modulation
    - Layer-based rendering system
    - User-defined mapping presets

- 🎥 Video export
    - Render animations directly to video
    - FFmpeg integration
    - Export individual frames

---

## Example Workflow

```
Audio File
      │
      ▼
Audio Analysis
      │
      ▼
Feature Extraction
      │
      ▼
Mapping Engine
      │
      ▼
Fractal Renderer
      │
      ▼
Real-time Visualisation / Video Export
```

---

## Technical Highlights

This project combines several challenging areas of software engineering:

- High-performance systems programming in Rust
- Digital signal processing
- Fractal mathematics
- Multi-threaded rendering
- Large-scale application architecture
- Interactive UI design

Several optimisation techniques were implemented to keep rendering responsive, including:

- Perturbation rendering
- Orbit rebasing
- Dynamic render quality
- Background rendering threads
- Cached reference orbits

---

## Performance

Rendering deep Mandelbrot zooms is computationally expensive.

To achieve interactive frame rates, the renderer incorporates:

- perturbation theory
- arbitrary precision reference orbits
- adaptive rendering quality
- background rendering workers
- cached calculations

Profiling (AMD uProf) identified rendering as the primary bottleneck, leading to several optimisation passes throughout development.

---

## What I Learned

This project gave me practical experience with:

- Designing large Rust applications
- Performance profiling and optimisation
- Concurrent programming
- Digital signal processing
- Numerical algorithms
- Building software from research papers

---

## Future Improvements

Some ideas for future development include:

- More advanced shader effects
- Machine learning generated mapping presets
- Additional fractal types
- Cross-platform binary releases

---

## Dissertation

This repository contains the implementation developed for my final-year dissertation at the University of Warwick.

The focus of the project was combining fractal rendering with music analysis to create an extensible, interactive visualisation platform while exploring performance optimisation techniques required for real-time rendering.

---

## How to run

### Prerequisites

It is recommended to build and run on Windows, as this software hasn't been tested on other operating systems, and some features are known to not work.

Before building the project, ensure you have the following installed:

- Rust (via `rustup` <https://rustup.rs/>)
- A working C toolchain (usually handled automatically by Rust on Windows)

### FFmpeg (Required, skip if using windows-x86_64)

This project depends on FFmpeg; development files (libraries and headers) are required for building. Static libraries are optional.

FFmpeg files for Windows are included under `ffmpeg/windows-x86_64` in this repository. Otherwise, download a compatible build manually:

- Recommended source: <https://www.gyan.dev/ffmpeg/builds/>

Download:

- **"release full"** or **"full_build"** version
- Ensure it includes development files (`.lib`, headers, etc.)

### Project Setup (skip if using windows x86-64)

Place the FFmpeg files in the following directory structure inside the repository:

```text
ffmpeg/windows-x86_64/
├── bin/
│ └── *.dll
├── lib/
│ └── *.lib
└── include/
```

> Note: windows-x86_64 is used here as an example

At minimum:

- `bin/` must contain FFmpeg DLLs (used at runtime)
- `lib/` and `include/` are required for static linking during build

### Environment Setup (Windows)

The project includes a PowerShell script (`scripts/cargo_runner.ps1`) that automatically prepends the FFmpeg `bin` directory to your `PATH` at runtime.

To ensure `scripts/cargo_runner.ps1` is used during the build process, this project includes a `.cargo/config.toml` to tell Cargo where to find FFmpeg and how to run the PowerShell script. This will have to be modified with values appropriate for your machine. The configuration expects Windows and a PowerShell runner that invokes `scripts/cargo_runner.ps1`.

**Config file location:** [.cargo/config.toml](.cargo/config.toml#L1-L6)

This repository includes a ready `.cargo/config.toml` and `scripts/cargo_runner.ps1` which Cargo will use when building and running on Windows. Edit `.cargo/config.toml` only if you need to change the runner or FFmpeg paths.

Example runner entry (already present in the provided config):

```toml
[target.'cfg(windows)']
runner = "powershell.exe -ExecutionPolicy Bypass -File .\\scripts\\cargo_runner.ps1"
```

After confirming the provided `.cargo/config.toml` is correct, run:

```bash
cargo run --release
```
