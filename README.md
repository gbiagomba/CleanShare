# CleanShare

## Background/Lore
`cleanshare` removes tracking parameters from URLs and unwraps common redirect wrappers, so you can safely share clean links. It works from the terminal, with piping, files, and multiple URLs, and supports custom rules via YAML/JSON.

## Table of Contents
- Features
- Installation
  - Using GitHub Releases
  - Using Cargo
  - Compiling from Source
- Flags
- Usage
  - Running Tests
  - Using Docker
  - Using the `Makefile`
- Contributing
- License

## Features
- Strip tracking params (utm_*, gclid, fbclid, msclkid, etc.).
- Unwrap known redirect wrappers (Google, Facebook, Reddit, YouTube).
- Input via pipe, `-u` args, or `-f` file; output to stdout or `-o` file.
- Extend behavior with YAML/JSON rules; host-specific and glob matching.

## Installation

### Using GitHub Releases
- Download the archive for your platform from the Releases page (Linux/macOS/Windows; x64/arm64).
- Extract and place `cleanshare` (or `cleanshare.exe`) in your `PATH`.

### Using Cargo
- `cargo install --git https://github.com/your-org-or-user/CleanShare.git` (or from crates.io when published)

### Compiling from Source
- Prereqs: Rust stable toolchain.
- Build: `cargo build --release`
- Binary: `target/release/cleanshare`

## Flags
- `-u, --url <URL>`: One or more URLs to clean (repeatable).
- `-f, --file <PATH>`: Read URLs (one per line) from file.
- `-o, --output <PATH>`: Write cleaned URLs to file instead of stdout.
- `-r, --rules <PATH>`: Load additional rules from YAML or JSON.
- `-v, --verbose`: Print non-fatal errors for invalid inputs.
- `-h, --help` / `-V, --version`: Show help/version.

## Usage
- Pipe from clipboard (macOS): `pbpaste | cleanshare`
- Clean a single URL: `cleanshare -u "https://example.com/?utm_source=x&gclid=123"`
- Multiple URLs: `cleanshare -u URL1 -u URL2`
- From a file: `cleanshare -f urls.txt`
- Output to file: `cleanshare -f urls.txt -o output.txt`
- Custom rules: `cleanshare -r rules.yaml -f urls.txt`

Rules format (YAML/JSON):
- `remove_params`: exact names to drop.
- `remove_param_globs`: glob patterns to drop (case-insensitive), e.g., `utm_*`, `ref*`.
- `keep_params`: params to keep even if matched elsewhere.
- `host_rules[]`: `{ hosts: ["*.domain"], unwrap_params: ["url"], remove_params, remove_param_globs, keep_params, strip_all_params }`.

See `rules.example.yaml` for a template.

### Running Tests
- `make test` or `cargo test`

### Using Docker
- Build image: `make docker-build`
- Run help: `make docker-run`
- Pipe URLs: `cat urls.txt | docker run --rm -i cleanshare:latest`

### Using the `Makefile`
- Build: `make build`
- Lint: `make lint`
- Format: `make fmt`
- Clean: `make clean`

## Contributing
- Open issues/PRs with clear descriptions. Add tests for changes. Keep code minimal, safe, and focused.

## License
CleanShare is licensed under the GPL-3.0 License. For more information, see the [LICENSE](LICENSE) file.
