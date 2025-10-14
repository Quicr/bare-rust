use clap::{Parser, Subcommand};
use hactar_cli::flasher::{flash, FlashArgs};
use hactar_cli::monitor::{monitor, MonitorArgs};
use std::process::exit;

#[derive(Parser)]
#[command(name = "hactar-cli")]
#[command(version, about = "Firmware flashing and serial monitoring tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Flash firmware to device
    Flash {
        /// COM/Serial port that Hactar is on, leave blank to search for Hactars
        #[arg(short, long, default_value = "")]
        port: String,

        /// Baudrate to communicate at
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,

        /// Chips that are to be flashed to. Available values: ui, net, mgmt.
        /// Multiple chips: ui+net, or ui+net+mgmt, etc
        #[arg(short, long)]
        chip: String,

        /// Path to the binary
        #[arg(long = "binary_path")]
        binary_path: Option<String>,

        /// Gets hactar into flashing mode and then exits so a 3rd party flasher can be used
        #[arg(short = 'e', long = "use_external_flasher", default_value_t = false)]
        use_external_flasher: bool,
    },

    /// Open serial monitor
    Monitor {
        /// Serial port
        #[arg(short, long, default_value = "")]
        port: String,

        /// Baudrate to communicate at
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Flash {
            port,
            baud,
            chip,
            binary_path,
            use_external_flasher,
        } => {
            let port = if port.is_empty() { None } else { Some(port) };

            flash(FlashArgs {
                port,
                baud,
                chip,
                binary_path,
                use_external_flasher,
            })
        }
        Commands::Monitor { port, baud } => {
            let port = if port.is_empty() { None } else { Some(port) };

            monitor(MonitorArgs { port, baud })
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
