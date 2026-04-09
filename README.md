# Accessing the source code

All source code can be found in the `src/` folder.

# How to run

## Prerequisites

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
