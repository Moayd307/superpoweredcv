
![SuperpoweredCV Banner](superpoweredcv-banner.png)

SuperpoweredCV is a comprehensive tool designed for red-teaming ATS (Applicant Tracking Systems) and AI resume parsers. It allows you to scrape LinkedIn profiles, generate PDF resumes with embedded prompt injections, and analyze how these systems interpret the data.

The project consists of three main components:
1.  **Core CLI**: A Rust-based command-line tool for analysis and PDF generation.
2.  **Browser Extension**: A Chrome/Firefox extension to scrape profile data.
## CLI Commands

The `superpoweredcv` CLI provides a suite of tools for generating, injecting, and analyzing resumes.

### General
*   `superpoweredcv docs`: Open the documentation in your default browser.
*   `superpoweredcv --help`: Show the help message and list of commands.

### Generation & Injection
*   `superpoweredcv generate --profile <json_path> --output <pdf_path>`: Generate a PDF resume from a scraped JSON profile.
    *   `--injection <type>`: Specify injection type (`VisibleMeta`, `LowVis`, `Offpage`, `TrackingPixel`, `CodeInjection`).
    *   `--intensity <level>`: Set intensity (`Soft`, `Medium`, `Aggressive`).
    *   `--position <pos>`: Set position (`Header`, `Footer`).
*   `superpoweredcv inject --input <pdf_path> --output <pdf_path> --type <type> --payload <content>`: Inject a payload into an existing PDF.
*   `superpoweredcv preview --output <pdf_path>`: Generate a preview PDF showing injection layouts.

### Analysis
*   `superpoweredcv analyze --scenario <path>`: Run an analysis scenario defined in a file.
*   `superpoweredcv demo`: Run the built-in demo scenario.
*   `superpoweredcv validate --config <path>`: Validate a configuration file.

## GUI Mode
Running `superpoweredcv` without arguments launches the graphical user interface.

### GUI Features
*   **Brutalist Design**: High-contrast, efficient interface.
*   **Independent Windows**: Settings, Logs, and Preview open in separate, pinnable windows.
*   **Visual Preview**: Real-time visualization of injection placement.
*   **LaTeX Builder**: Visual builder for resume sections.

## Getting Started

### 1. Install the Browser Extension
The extension allows you to easily grab profile data from LinkedIn to use as a base for your experiments.

*   **Chrome**: Load the `extension/` folder as an unpacked extension in `chrome://extensions`.
*   **Firefox**: Load `extension/manifest-firefox.json` as a temporary add-on in `about:debugging`.

See [extension/README.md](extension/README.md) for detailed instructions.

### 2. Build the CLI
The core logic is written in Rust. You'll need a Rust toolchain installed.

**Important:** All cargo commands must be run from the `core` directory.

```bash
cd core
cargo build --release
```

## Usage

The CLI provides several commands to manage the workflow from data ingestion to report generation.

### Generate a PDF Resume
Convert a scraped JSON profile into a PDF. This is useful for creating a baseline resume before applying injections.

```bash
cd core
cargo run -- generate --file <path/to/profile.json> --output resume.pdf
```

### Run an Analysis Scenario
Execute a red-teaming scenario defined in a configuration file. This simulates how an ATS might parse the resume with various injections.

```bash
cd core
cargo run -- analyze --scenario <path/to/scenario.yaml>
```

### Run the Demo
Run a built-in demo scenario to see the tool in action.

```bash
cd core
cargo run -- demo
```

### Validate Configuration
Check if your configuration files are valid.

```bash
cd core
cargo run -- validate --config <path/to/config.yaml>
```

## Project Structure

- `core/`: Rust CLI and library.
    - `src/profile.rs`: Data structures for profiles.
    - `src/generator.rs`: PDF generation logic.
    - `src/red_team.rs`: Injection and scenario logic.
- `extension/`: Browser extension source code.
- `docs/`: Documentation and specifications.

## Development

To run the project in development mode:

```bash
cd core
cargo run -- help
```
