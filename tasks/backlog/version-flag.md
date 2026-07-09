# Add --version flag

## Requirements

`ash --version` should print the current version of ash and exit.

- Accept `--version` (long form) and `-V` (short form, standard convention)
- Print version string: `ash <version>` (e.g. `ash 0.1.0`)
- Read version from `Cargo.toml` at build time
- Exit with code 0 after printing
