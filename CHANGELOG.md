# Changelog

## [0.1.10] - UNRELEASED
### Changed
- Reduce dependencies and thus compile time.
- Make expression table scrollable.
- Reduce updates to expression table during editing.
### Added
- Add cli history search.
- Make layout configurable.
- Add !layout command.
- Add ability to change expression integer format in expression table.
- Add hotkey (ctrl-w) for adding watchpoints for expressions in expression table.
- Add !show command to explicitly load files in the pager.
- Add show_file ipc command

## [0.1.9] - 2020-11-14
### Changed
- Generate gdb completions dynamically.
### Added
- Add rr support.
- Allow initial expression table entries to be specified using -e.

## [0.1.8] - 2020-07-15
### Changed
- Remove libgit2 build dependency.
### Fixed
- Compilation failing for arm-unknown-linux-* (via dependency).
- Bold style not reseting on some terminals.

## [0.1.7] - 2020-04-03
### Fixed
- Breakpoint message parsing for newer versions of gdb.
### Added
- Warning when trying to disassemble source files when gdb is busy.

## [0.1.6] - 2020-02-03
### Fixed
- Source view would sometimes still show an outdated version of the displayed file.
- Crash when failing to spawn gdb process (#13).

## [0.1.5] - 2019-10-24
### Added
- Tab completion support in console and expression table.

## [0.1.4] - 2019-07-21
### Fixed
- Incorrect background color selection (#4).
- Crash due to out-of-bounds write to terminal.
- Avoid pager consuming all screen space on long file or function names (#8).

## [0.1.3] - 2019-04-04
### Changed
- Always show stack frame info above pager (not only for source code).
### Fixed
- Unexpected switches between src/asm modes in pager (#1).
- Decrease wait time when stepping/toggling pager modes by only recomputing pager content when necessary.
- Avoid crash in case of out-of-order gdbmi responses.

## [0.1.2] - 2019-03-31
### Changed
- Allow publication on crates.io

## [0.1.1] - 2019-03-24
### Changed
- Fix building outside of the git repository.

## [0.1.0] - 2019-03-23
### Added
- Initial release.
