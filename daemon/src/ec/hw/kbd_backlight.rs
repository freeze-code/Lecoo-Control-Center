use ipc::KeyboardBacklightLevel;
use anyhow::Result;
use super::EcDevice;

const KEYBOARD_BACKLIGHT_REG: u16 = 0x0F05;

pub fn read_keyboard_backlight(ec: &EcDevice) -> Result<KeyboardBacklightLevel> {
    match ec.read_reg(KEYBOARD_BACKLIGHT_REG)? {
        0x00 => Ok(KeyboardBacklightLevel::Off),
        0x01 => Ok(KeyboardBacklightLevel::Low),
        0x02 => Ok(KeyboardBacklightLevel::Medium),
        0x03 => Ok(KeyboardBacklightLevel::High),
        v => Err(anyhow::anyhow!("Invalid keyboard backlight level: {:#04x}", v)),
    }
}

pub fn apply_keyboard_backlight(ec: &EcDevice, level: &KeyboardBacklightLevel) -> Result<()> {
    let mut addr = KEYBOARD_BACKLIGHT_REG;
    if ec.hram_offset == 0xC400 {
        // WORKAROUND for EC base offset 0xC400
        addr += 0xC000
    }

    ec.write_reg(addr, *level as u8)?;
    Ok(())
}
