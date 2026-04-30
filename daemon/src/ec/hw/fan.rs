use anyhow::{Result, bail};
use ipc::{FanIndex, FanMode};
use super::EcDevice;

pub fn apply_fan_mode(ec: &EcDevice, fan: &FanIndex, mode: &FanMode) -> Result<()> {
    let thermal_policy_override: u16 = match fan {
        FanIndex::Cpu => ec.offsets.ram_thermal_policy_cpu,
        FanIndex::Gpu => ec.offsets.ram_thermal_policy_gpu,
    };

    match mode {
        FanMode::Auto => {
            ec.write_ram(thermal_policy_override, 0x00)?;
            ec.write_ram(*fan as u16, 0)?;
        }
        FanMode::Full => {
            ec.write_ram(thermal_policy_override, 0x40)?;
            ec.write_ram(*fan as u16, 150)?;

        }
        FanMode::Custom(duty) => {
            if *duty > 220 {
                bail!("Duty cycle too high, it's dangerous!"); // todo: custom message, not error type! replace
            }

            ec.write_ram(thermal_policy_override, 0x40)?;
            ec.write_ram(*fan as u16, *duty)?;
        }
    };

    Ok(())
}
