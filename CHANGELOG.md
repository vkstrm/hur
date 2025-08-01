## Unreleased 

### Added

- `req` subcommand, for making requests.
- `config` subcommand, for creating a config file.
- `--body-file` flag for passing a file as body.
- `--redirect-mode` flag to choose if to follow redirects or not.

### Changed

- **Breaking** Refactored CLI into subcommands `req` and `config`.
- Using Clap derive, changing help messages and behaviour.
- Change to 2024 edition.
- `--headers` to `--headers-json`.
- `--info` to `--debug`

### Removed

- `--no-body` flag.

## v0.1.0

First release
