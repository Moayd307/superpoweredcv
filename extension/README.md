# SuperpoweredCV Profile Grabber

This is a browser extension to scrape LinkedIn profiles and export them as JSON for SuperpoweredCV analysis. It supports both Google Chrome and Mozilla Firefox.

## Installation

### Google Chrome
1. Open Chrome and navigate to `chrome://extensions`.
2. Enable "Developer mode" in the top right corner.
3. Click "Load unpacked".
4. Select this `extension` directory.

### Mozilla Firefox
1. Open Firefox and navigate to `about:debugging#/runtime/this-firefox`.
2. Click "Load Temporary Add-on...".
3. Select the `manifest-firefox.json` file in this directory.

## Usage

1. Navigate to a LinkedIn profile page (e.g., `https://www.linkedin.com/in/your-profile`).
2. Click the SuperpoweredCV extension icon in the toolbar.
3. Click "Grab Profile".
4. A JSON file (`profile_Name.json`) will be downloaded.

## Next Steps

Use the SuperpoweredCV CLI to generate a PDF resume from the downloaded JSON:

```bash
cd ../core
cargo run -- generate --file <path/to/profile.json> --output resume.pdf
```
