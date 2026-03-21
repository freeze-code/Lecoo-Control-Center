use ipc::ChargeLimit;
use anyhow::{Ok, Result};
use super::EcDevice;

const RAM_BAT_RSOC: u16 = 0x93;
const RAM_BAT_LIMIT_MIN: u16 = 0xBC;
const RAM_BAT_LIMIT_MAX: u16 = 0xBB;

pub fn read_charge_limit(ec: &EcDevice) -> Result<(u8, u8)> {
    let min = ec.read_ram(RAM_BAT_LIMIT_MIN)? as u8;
    let max = ec.read_ram(RAM_BAT_LIMIT_MAX)? as u8;
    Ok((min, max))
}

pub fn read_battery_rsoc(ec: &EcDevice) -> Result<u8> {
    ec.read_ram(RAM_BAT_RSOC)
}

pub fn apply_charge_limit(ec: &EcDevice, limit: &ChargeLimit) -> Result<()> {
    let (min, max) = limit.as_percent();

    ec.write_ram(RAM_BAT_LIMIT_MIN, min)?;
    ec.write_ram(RAM_BAT_LIMIT_MAX, max)?;

    Ok(())
}
