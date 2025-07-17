# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0](https://github.com/jmjoy/fastcgi-client-rs/compare/v0.9.0...v0.10.0) - 2025-07-15

### Added

- add release workflow for automated package releases ([#24](https://github.com/jmjoy/fastcgi-client-rs/pull/24))
- reimplement request-stream with async Stream trait ([#20](https://github.com/jmjoy/fastcgi-client-rs/pull/20))

### Other

- Update documents ([#23](https://github.com/jmjoy/fastcgi-client-rs/pull/23))
- reduce overall dependencies by using sub-futures crates ([#22](https://github.com/jmjoy/fastcgi-client-rs/pull/22))
- Add ability to custom insertion in Params ([#18](https://github.com/jmjoy/fastcgi-client-rs/pull/18))
- Update workflow configurations to use ubuntu-latest ([#21](https://github.com/jmjoy/fastcgi-client-rs/pull/21))
- Refactor execute_once_stream function signature and remove redundant comment in conn.rs
