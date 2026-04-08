# Font Extractor

[中文](README.md) | English

A desktop application built with Tauri + SolidJS for extracting font subsets based on specified characters, effectively reducing font file size.

## Features

- **Font Subsetting**: Extract font files containing only the characters needed from input text
- **TTC Support**: Extract specific fonts from TrueType Collection files by index
- **Live Preview**: Preview font rendering after entering text
- **Cross-Platform**: Supports Windows, macOS, and Linux

## Tech Stack

- **Frontend**: SolidJS + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Font Processing**: allsorts, ttf-parser

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+

### Install Dependencies

```bash
npm install
```

## Development

```bash
# Start development server
npm run tauri:dev
```

## Build

```bash
# Build for current platform
npm run tauri:build

# Build for Windows
npm run tauri:build:win

# Build for macOS (must run on macOS)
npm run tauri:build:mac
```

After building, installers are located in `src-tauri/target/release/bundle/`.

## Usage

1. Open the app and click "Select Font File" to upload a font (supports TTF, OTF, TTC formats)
2. Enter the characters you want to keep in the text input
3. Click the "Extract Font" button
4. Choose a save location and wait for processing to complete

## Project Structure

```
.
├── src/                    # Frontend source (SolidJS)
│   ├── components/         # Components
│   ├── App.tsx            # Main app component
│   └── index.tsx          # Entry file
├── src-tauri/             # Tauri backend (Rust)
│   ├── src/
│   │   └── lib.rs         # Core font processing logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── index.html             # HTML template
├── vite.config.ts         # Vite configuration
└── package.json           # Node.js dependencies
```

## Core Implementation

### Font Subsetting

Uses Rust's `allsorts` library for font subset extraction:

1. Parse input text to extract unique character set
2. Map characters to glyph IDs via cmap table
3. Use `SubsetProfile::Minimal` for minimal subset extraction
4. Generate new font file containing only required glyphs

### TTC Font Processing

Supports extracting individual fonts from TrueType Collection:

1. Parse TTC header to get font offset tables
2. Extract table data for the specified font index
3. Rebuild independent TTF file structure
4. Recalculate checksums to ensure font validity

## License

MIT
