# Version 0.3.0 (January 10th, 2025)

- Update dependency `cargo_metadata` from `0.15` to `0.19`.
- Update dependency `crossterm` from `0.25` to `0.28`.
- Add carriage return to start of message line to fix messages displaying incorrectly.

# Version 0.2.1 (August 26th, 2022)

- Allow messages without error codes with a span to be displayed.

# Version 0.2.0 (August 25th, 2022)

- Added new options
  - `-V` / `--version`: display burden and cargo version.
  - `--working-dir`: set the directory that cargo runs in.
  - `--color`: Specify `--color` used by cargo.
- `--help` / `-h` now only displays help for burden if before the subcommand.
  - After the subcommand it will display the subcommand help from cargo.
- Using `--color` or `--message-format` after the subcommand will now display a warning as it is overrided by burden.
- Stdout will now display after using `Esc` or `Ctrl^C` if using the `run` subcommand.
- Messages that don't have a code are now omitted.
  - Examples: `warning: 2 warnings emitted` or `error: aborting due to previous error`
- Updated `crossterm: 0.24 -> 0.25`.

# Version 0.1.1 (August 5th, 2022)

- Show cursor and disable raw mode on exit.

# Version 0.1.0 (August 5th, 2022)

- Initial Release
