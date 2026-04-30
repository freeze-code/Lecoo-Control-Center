use anyhow::Result;
use super::EcDevice;

pub fn read_system_info(ec: &EcDevice) -> Result<(u8, u8, u8)> {
    let offset = ec.offsets;
    let chip_id1 = ec.read_reg(offset.reg_chip_id1)?;
    let chip_id2 = ec.read_reg(offset.reg_chip_id2)?;
    let chip_ver = ec.read_reg(offset.reg_chip_ver)?;
    Ok((chip_id1, chip_id2, chip_ver))
}

pub fn read_fans_rpm(ec: &EcDevice) -> Result<(u16, u16)> {
    let offset = ec.offsets;
    let cpu_msb = ec.read_ram(offset.ram_fan_cpu_msb)? as u16;
    let cpu_lsb = ec.read_ram(offset.ram_fan_cpu_lsb)? as u16;
    let cpu_rpm = ((cpu_msb << 8) | cpu_lsb) as u16;

    let gpu_msb = ec.read_ram(offset.ram_fan_gpu_msb)? as u16;
    let gpu_lsb = ec.read_ram(offset.ram_fan_gpu_lsb)? as u16;
    let gpu_rpm = ((gpu_msb << 8) | gpu_lsb) as u16;

    Ok((cpu_rpm, gpu_rpm))
}

pub fn read_temperatures(ec: &EcDevice) -> Result<(u8, u8)> {
    let offset = ec.offsets;
    let cpu_temp = ec.read_ram(offset.ram_temp_cpu)? as u8;
    let sys_temp = ec.read_ram(offset.ram_temp_sys)? as u8;
    Ok((cpu_temp, sys_temp))
}
