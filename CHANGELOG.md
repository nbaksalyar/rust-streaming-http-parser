# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) 
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.3.2] - 2018-01-15
### Changed
- Replace the deprecated `gcc` dependency with `cc` in the build script

## [0.3.1] - 2016-12-23
### Added
- `pause` and `unpause` methods calling underlying `http_parser_pause` (thanks to @3Hren)

## [0.3.0] - 2016-10-11
### Changed
- Breaking API change: now all callbacks receive a mutable reference to parser
- Node.js HTTP parser updated to 2.7.1
- Example moved from readme to a separate file
