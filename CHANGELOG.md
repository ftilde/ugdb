# Changelog

## [Unreleased]
### Changed
- Always show stack frame info above pager (not only for source code).
### Fixed
- Unexpected switches between src/asm modes in pager (#1).
- Decrease wait time when stepping/toggling pager modes by only recomputing pager content when necessary.
- Avoid crash in case of out-of-order gdbmi responses.

## [0.1.2] - 2019-03-24
### Changed
- Allow publication on crates.io

## [0.1.1] - 2019-03-24
### Changed
- Fix building outside of the git repository.

## [0.1.0] - 2019-03-23
### Added
- Initial release.
