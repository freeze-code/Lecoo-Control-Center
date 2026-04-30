use std::io::{self, Write};

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use ipc::{ChargeLimit, FanIndex, FanMode, IpcClient, IpcRequest, IpcResponse, KeyboardBacklightLevel, PowerLedMode, PowerProfile};

#[derive(Parser)]
#[command(name = "lecoo-ctrl")]
#[command(version, about = "Lecoo Control Center CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get info about the embedded controller
    Info,

    /// Get system temperatures
    Temps,

    /// Get current fan speeds (RPM)
    Fans,

    /// Get real-time monitoring of temps and fans
    Monitoring {
        rate: Option<f32>
    },

    /// Control fan settings (e.g., `fan cpu auto`, `fan gpu custom 150`)
    Fan {
        #[arg(value_enum)]
        target: CliFanIndex,

        #[arg(value_enum)]
        mode: CliFanMode,

        val: Option<u8>,
    },

    /// Get or set battery charge limit
    Charge {
        #[arg(value_enum)]
        limit: Option<CliChargeLimit>,
    },

    /// Set system power profile
    Power {
        #[arg(value_enum)]
        profile: Option<CliPowerProfile>,
    },

    /// Set keyboard backlight level
    Kbd {
            #[arg(value_enum)]
            mode: Option<CliKbdMode>,

            val: Option<u8>,
        },

    /// Control rear LED ring (e.g., `led auto`, `led custom 255`)
    Led {
        #[command(subcommand)]
        action: CliLedAction,
    },

    /// Control daemon settings, only for advanced users
    Daemon {
        #[command(subcommand)]
        action: CliDaemonAction,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliPowerProfile {
    Silent,
    Default,
    Perf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliFanIndex {
    Cpu,
    Gpu,
    Both,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliFanMode {
    Auto,
    Full,
    Custom,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliKbdMode {
    Off,
    Low,
    Medium,
    High,
    Custom,
}

#[derive(Subcommand, Clone)]
enum CliLedAction {
    /// Let EC control the LED automatically
    Auto,
    /// Set manual static brightness (0-255)
    Custom { val: u8 },
    // todo: add others
}

#[derive(Clone, Subcommand)]
enum CliDaemonAction {
    /// Change telemetry settings
    Telemetry {
        #[command(subcommand)]
        telemetry_action: CliTelemetryAction
    },

    /// Change daemon settings
    Settings {
        #[command(subcommand)]
        settings_action: CliSettingsAction
    },

    /// Get daemon version
    Version,
}

#[derive(Subcommand, Clone)]
enum CliTelemetryAction {
    /// Enable telemetry
    Enable,
    /// Disable telemetry
    Disable,
    /// Get telemetry ID
    Id,
}

#[derive(Subcommand, Clone)]
enum CliSettingsAction {
    /// Reset daemon settings to default
    Reset,
    /// Read daemon settings
    Read,
    /// Apply saved settings state
    Apply,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliChargeLimit {
    /// Full capacity (Charges to 100%)
    Full,
    /// High capacity (Charges to 95%)
    High,
    /// Balanced mode (Charges to 80%)
    Balanced,
    /// Maximum battery lifespan (Charges to 60%)
    Lifespan,
    /// Desk mode for plugged-in usage (Charges to 40%)
    Desk,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut client = IpcClient::connect().context("Failed to connect to daemon! Is it running?")?;

    let request = match cli.command {
        Commands::Info => IpcRequest::GetSystemState,
        Commands::Temps => IpcRequest::GetTemperatures,
        Commands::Fans => IpcRequest::GetFansRPM,


        Commands::Monitoring { rate } => {
            let update_rate = (rate.unwrap_or(1.0) * 1000.0) as u64;
            println!("Monitoring enabled. Update rate: {} ms", update_rate);
            loop {
                let IpcResponse::Temp(cpu, system) = client.request(&IpcRequest::GetTemperatures)? else { unreachable!() };
                let IpcResponse::FanRPM(cpu_fan, gpu_fan) = client.request(&IpcRequest::GetFansRPM)? else { unreachable!() };

                print!("\r🌡️ CPU: {}°C, Sys: {}°C | 💨 Fans: {} RPM (CPU), {} RPM (GPU)      ",
                    cpu, system, cpu_fan, gpu_fan
                );
                io::stdout().flush().unwrap();

                std::thread::sleep(
                    std::time::Duration::from_millis(update_rate)
                );
            }
        }

        Commands::Power { profile } => match profile {
            None => IpcRequest::GetPowerProfile,
            Some(p) => {
                let power_profile = match p {
                    CliPowerProfile::Silent => PowerProfile::Silent,
                    CliPowerProfile::Default => PowerProfile::Default,
                    CliPowerProfile::Perf => PowerProfile::Performance,
                };
                IpcRequest::SetPowerProfile(power_profile)
            }
        }

        Commands::Fan { target, mode, val } => {
            let fan_mode = match mode {
                CliFanMode::Auto => FanMode::Auto,
                CliFanMode::Full => FanMode::Full,
                CliFanMode::Custom => FanMode::Custom(val.unwrap_or(0)),
            };
            let fan_idx = match target {
                CliFanIndex::Cpu => FanIndex::Cpu,
                CliFanIndex::Gpu => FanIndex::Gpu,
                CliFanIndex::Both => {
                    let _: IpcResponse = client.request(&IpcRequest::SetFanMode { fan: FanIndex::Cpu, mode: fan_mode } )?;
                    FanIndex::Gpu
                },
            };
            IpcRequest::SetFanMode { fan: fan_idx, mode: fan_mode }
        }

        Commands::Charge { limit } => match limit {
            Some(cli_limit) => {
                let ipc_limit = match cli_limit {
                    CliChargeLimit::Full => ChargeLimit::FullCapacity,
                    CliChargeLimit::High => ChargeLimit::HighCapacity,
                    CliChargeLimit::Balanced => ChargeLimit::Balanced,
                    CliChargeLimit::Lifespan => ChargeLimit::MaximumLifespan,
                    CliChargeLimit::Desk => ChargeLimit::DeskMode,
                };
                IpcRequest::SetChargeLimit(ipc_limit)
            }
            None => IpcRequest::GetChargeLimit,
        },

        Commands::Kbd { mode, val } => match mode {
            Some(m) => {
                let lvl = match m {
                    CliKbdMode::Off => KeyboardBacklightLevel::Off,
                    CliKbdMode::Low => KeyboardBacklightLevel::Low,
                    CliKbdMode::Medium => KeyboardBacklightLevel::Medium,
                    CliKbdMode::High => KeyboardBacklightLevel::High,
                    CliKbdMode::Custom => KeyboardBacklightLevel::Custom(val.unwrap_or(255)),
                };
                IpcRequest::SetKeyboardBacklight(lvl)
            }
            None => IpcRequest::GetKeyboardBacklight,
        },

        Commands::Led { action } => {
            let led_m = match action {
                CliLedAction::Auto => PowerLedMode::Auto,
                CliLedAction::Custom { val } => PowerLedMode::Custom(val), // todo
            };
            IpcRequest::SetLedMode(led_m)
        }

        Commands::Daemon { action } => match action {
            CliDaemonAction::Telemetry { telemetry_action } => match telemetry_action {
                CliTelemetryAction::Enable => IpcRequest::DaemonCommand(ipc::DaemonCommand::ActivateTelemetry(true)),
                CliTelemetryAction::Disable => IpcRequest::DaemonCommand(ipc::DaemonCommand::ActivateTelemetry(false)),
                CliTelemetryAction::Id => IpcRequest::DaemonCommand(ipc::DaemonCommand::GetTelemetryId),
            },
            CliDaemonAction::Settings { settings_action } => match settings_action {
                CliSettingsAction::Reset => IpcRequest::DaemonCommand(ipc::DaemonCommand::RestoreDefaults),
                CliSettingsAction::Read => IpcRequest::DaemonCommand(ipc::DaemonCommand::GetSettings),
                CliSettingsAction::Apply => IpcRequest::DaemonCommand(ipc::DaemonCommand::ApplySettings),
            },
            CliDaemonAction::Version => {
                println!("{}.{}", client.daemon_version.0, client.daemon_version.1);
                std::process::exit(0);
            },
        },
    };

    // --------------

    let res: IpcResponse = client.request(&request)?;

    // Result handling
    match res {
        IpcResponse::Success => println!("Done."),

        IpcResponse::SystemInfo(chip_name, chip_rev, hram_offset, version) => {
            println!("Controller: {} (Rev {}) \nHRAM Offset: 0x{:04X} \nDaemon Version: {}",
                chip_name, chip_rev, hram_offset, version
            );
        }

        IpcResponse::FanRPM(cpu, gpu) => {
            println!("⚙️ Fan Speeds:");
            println!("   CPU: {} RPM", cpu);
            println!("   GPU: {} RPM", gpu);
        }

        IpcResponse::Temp(cpu, system) => {
            println!("🌡️ Temperatures:");
            println!("   CPU: {} °C", cpu);
            println!("   System: {} °C", system);
        }

        IpcResponse::KeyboardBacklight(level) => {
            println!("💡 Keyboard Backlight:");
            println!("   Level: {}", level);
        }

        IpcResponse::ChargeLimit(min, max, current) => {
            println!("🔋 Charge Limit (FlexiCharger):");
            if min == 0 && max == 0 {
                println!("   Mode: Full Capacity (Charges to 100%)");
            } else {
                println!("   Start charging at: {}%", min);
                println!("   Stop charging at:  {}%", max);
            }
            println!("   Current Battery Charge: {}%", current);
        }

        IpcResponse::PowerLimit(profile) => {
            println!("⚡ Power Profile:");
            println!("   Current: {}", profile);
        }

        IpcResponse::TelemetryDisabledInfo => {
            println!("🔗 Anonymous Telemetry Disabled. Telemetry helps me improve the quality of this project.\nPlease consider enabling it, it's free and anonymous :)");
        }

        IpcResponse::Error(msg) => {
            eprintln!("❌ Error: {}", msg);
            std::process::exit(1);
        }

        IpcResponse::DaemonResponse(daemon_response) => match daemon_response {
            ipc::DaemonResponse::Settings(settings) => println!("{:#?}", settings),
            ipc::DaemonResponse::TelemetryId(id) => {
                println!("🔗 Telemetry ID: 0x{:016X}", id);
            },
        },
    }

    Ok(())
}
