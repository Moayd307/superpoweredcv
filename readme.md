
![SuperpoweredCV Banner](superpoweredcv-banner.png)

SuperpoweredCV is a comprehensive tool designed for red-teaming ATS (Applicant Tracking Systems) and AI resume parsers. It allows you to scrape LinkedIn profiles, generate PDF resumes with embedded prompt injections, and analyze how these systems interpret the data.

The project consists of three main components:
1.  **Core CLI**: A Rust-based command-line tool for analysis and PDF generation.
2.  **Browser Extension**: A Chrome/Firefox extension to scrape profile data.
3.  **Documentation**: Guides and specifications.

## Getting Started

### 1. Install the Browser Extension
The extension allows you to easily grab profile data from LinkedIn to use as a base for your experiments.

*   **Chrome**: Load the `extension/` folder as an unpacked extension in `chrome://extensions`.
*   **Firefox**: Load `extension/manifest-firefox.json` as a temporary add-on in `about:debugging`.

See [extension/README.md](extension/README.md) for detailed instructions.

### 2. Build the CLI
The core logic is written in Rust. You'll need a Rust toolchain installed.

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
