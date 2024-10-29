# Song Transition Generator

## Introduction
This project demonstrates how to create a DJ-style transition between two WAV files using Rust. The code reads two WAV files, adjusts the tempo of the second song to match the first, and then creates a smooth crossfade transition between them.

## Features
- Load and read WAV files
- Check and handle different channel configurations (mono and stereo)
- Adjust the tempo of the second song to match the first using the `rubato` crate
- Create a crossfade transition between the two songs
- Write the resulting transition to a new WAV file

## Dependencies
- `dasp`: Digital Audio Signal Processing library
- `hound`: WAV encoding and decoding library
- `rubato`: High-quality resampling library
- `std`: Standard library for Rust

## Code Structure
- `main.rs`: The main file containing the logic for reading, processing, and writing WAV files

## Usage
1. Ensure you have Rust installed on your system.
2. Clone the repository.
3. Build the project using `cargo build`.
4. Run the project using `cargo run`, ensuring that you have the two input wav files in the project directory.
