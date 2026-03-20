# Changelog

## [0.3.1] - 2026-03-20

### Added
- Battery charge limits are now actively applied when loading daemon settings.
- Added `RpcSs` as a dependency in the Windows installation script (`install.bat`) to ensure more reliable service startup.

### Changed
- CPU and OS information on Windows is now fetched directly from the Windows Registry instead of using `raw_cpuid`.
- Refactored the Windows daemon startup flow to explicitly wait for the OS service initialization to complete before binding the IPC server and initializing EC communication, preventing potential race conditions.

### Fixed
- Fixed a bug in EC HRAM window base address detection by correctly treating `0xFF` (instead of `0`) as the uninitialized offset.
- Prevented the daemon from crashing (panicking) during system shutdown if reading the keyboard backlight state fails.
- Ensure the active power profile is correctly read and saved alongside the keyboard backlight state during system shutdown.
- Fixed the Windows uninstaller script (`uninstall.bat`) attempting to remove the wrong service name (`LecooControlCenter` instead of `LecooControlDaemon`).