# MOV to MP4 Converter

A simple command-line tool to convert MOV files to MP4 format using FFmpeg.

## Features

- Converts MOV files to MP4 format using FFmpeg
- Shows real-time progress with elapsed time
- Option to delete original MOV files after conversion
- Detailed error reporting for failed conversions
- Cross-platform support (Windows, macOS, Linux)

## Prerequisites

- FFmpeg must be installed on your system. You can install it in one of two ways:
  1. **System-wide installation (Recommended)**:
     - Windows: Download from [FFmpeg website](https://ffmpeg.org/download.html) and add to PATH
     - macOS: `brew install ffmpeg`
     - Linux: `sudo apt install ffmpeg` (Ubuntu/Debian) or equivalent for your distribution
  2. **Local installation**: Place FFmpeg executable in the `bin/ffmpeg` directory:
     - Windows: `bin/ffmpeg/ffmpeg.exe`
     - macOS/Linux: `bin/ffmpeg/ffmpeg`

## Installation

1. Download the latest release for your operating system from the releases page
2. Extract the archive to your desired location

## Usage

1. Create a `mov` directory in the same location as the executable
2. Place your MOV files in the `mov` directory
3. Run the program:
   ```bash
   ./mov_to_mp4
   ```
4. When prompted, type `yes` or `y` if you want to delete the original MOV files after conversion, or any other input to keep them.

The converted MP4 files will be saved in a `mp4` directory.

## Building from Source

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Clone this repository
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The executable will be in `target/release/mov_to_mp4`

## Building for Windows

To build for Windows from macOS or Linux:

1. Add the Windows target:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```
2. Install the MinGW toolchain (macOS: `brew install mingw-w64`, Linux: `sudo apt install mingw-w64`)
3. Build the Windows executable:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```
4. The Windows executable will be in `target/x86_64-pc-windows-gnu/release/mov_to_mp4.exe`