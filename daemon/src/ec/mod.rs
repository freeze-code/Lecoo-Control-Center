use anyhow::{bail, Result};
use std::sync::Mutex;

#[cfg(target_os = "linux")]
mod sys_linux;
#[cfg(target_os = "linux")]
pub use sys_linux::RawPortIo;

#[cfg(target_os = "windows")]
mod sys_windows;
#[cfg(target_os = "windows")]
pub use sys_windows::RawPortIo;

mod hw;
pub use hw::*;
mod offsets;
pub use offsets::*;

/// Platform-independent Embedded Controller hardware interface
pub struct EcDevice {
    /// Mutex wraps the low-level I/O backend.
    /// Locking it ensures atomic multi-step Super I/O transactions,
    /// preventing thread race conditions during Index/Data port writes.
    io: Mutex<RawPortIo>,
    /// Detected Super I/O base port
    port: u16,
    pub offsets: EcOffsets,
    pub hram_offset: u16,
}

impl EcDevice {
    /// Initializes the EC interface and auto-detects the active port.
    pub fn new() -> Result<Self> {
        // Initialize the platform-specific low-level I/O
        let io = RawPortIo::new()?;

        let mut offsets = EcOffsets::DEFAULT_N155A;
        let motherboard = crate::services::get_board_name();

        // Probe for motherboard type
        if motherboard.contains("N155A") {
            log::info!("Detected motherboard N155A.");
        }
        else if motherboard.contains("N155C") {
            log::info!("Detected motherboard N155C.");
        }
        else if motherboard.contains("N155D") {
            log::info!("Detected motherboard N155D.");
            offsets = EcOffsets::DEFAULT_N155D;
        } else {
            // todo: check insecure mode
            // bail!("Unsupported motherboard: {}", motherboard);
            log::error!("Unsupported motherboard: {}", motherboard);
            log::error!("Be careful. This will panic in future updates!");
        }

        let mut device = Self {
            io: Mutex::new(io),
            port: 0,
            offsets,
            hram_offset: 0xFF
        };

        device.probe_chip()?;

        let possible_bases: [u16; 5] = [0xC400, 0xC000, 0x0400, 0x0000, 0xE000];
        for &base in &possible_bases {
            // A REALLY(!) weak heuristic for detecting HRAM window
            if let Ok(temp) = device.read_reg(base + device.offsets.ram_temp_cpu) {
                if temp > 0x10 && temp < 0x50 {
                    device.hram_offset = base;
                    log::info!("HRAM Window detected by offset: {:#06X}. Temp: {}", base, temp);
                    break;
                }
            }
        }

        if device.hram_offset == 0xFF {
            bail!("Failed to detect HRAM window base address");
        }

        if device.hram_offset == 0xC400 {
            log::info!("EC base offset is 0xC400. Adjusting register offsets.");
            device.offsets.reg_kbd_backlight += 0xC000;
        }

        Ok(device)
    }

    /// Probes common Super I/O ports to find the ITE chip.
    fn probe_chip(&mut self) -> Result<()> {
        let probe_ports = [0x2E, 0x4E, 0x6E];

        for &p in &probe_ports {
            self.port = p;

            if let Ok(chip_id) = self.read_reg(0x2000) {
                if chip_id == 0x55 {
                    return Ok(()); // Successfully found IT5570
                }
                if chip_id == 0x81 || chip_id == 0x85 || chip_id == 0x89 || chip_id == 0x90 {
                    log::warn!("Warning: Found chip ID {:#X} at port {:#X}", chip_id, self.port);
                    log::warn!("Note: This chip may not be fully supported");
                    return Ok(());
                }
            }
        }

        // TODO: add insecure mode!
        bail!("ITE IT5570/IT8987 chip not found on any known port")
    }

    // --- High-Level EC Access ---

    /// Reads a single byte from the specified EC register address.
    pub fn read_reg(&self, addr: u16) -> Result<u8> {
        // Lock the mutex for the entire 6-step transaction.
        // No other thread can access `io.outb` or `io.inb` until this block finishes.
        let io = self.io.lock().unwrap();
        let port = self.port;

        let addr_high = (addr >> 8) as u8;
        let addr_low = (addr & 0xFF) as u8;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x11)?;

        io.outb(port, 0x2F)?;
        io.outb(port + 1, addr_high)?;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x10)?;

        io.outb(port, 0x2F)?;
        io.outb(port + 1, addr_low)?;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x12)?;

        io.outb(port, 0x2F)?;
        io.inb(port + 1)
    }

    /// Writes a single byte to the specified EC register address.
    pub fn write_reg(&self, addr: u16, val: u8) -> Result<()> {
        let io = self.io.lock().unwrap();
        let port = self.port;

        let addr_high = (addr >> 8) as u8;
        let addr_low = (addr & 0xFF) as u8;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x11)?;

        io.outb(port, 0x2F)?;
        io.outb(port + 1, addr_high)?;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x10)?;

        io.outb(port, 0x2F)?;
        io.outb(port + 1, addr_low)?;

        io.outb(port, 0x2E)?;
        io.outb(port + 1, 0x12)?;

        io.outb(port, 0x2F)?;
        io.outb(port + 1, val)?;

        Ok(())
    }

    // --- Hardware-Specific Helpers (Shared Memory Space) ---

    pub fn read_ram(&self, offset: u16) -> Result<u8> {
        self.read_reg(self.hram_offset + offset)
    }

    pub fn write_ram(&self, offset: u16, val: u8) -> Result<()> {
        self.write_reg(self.hram_offset + offset, val)
    }
}
