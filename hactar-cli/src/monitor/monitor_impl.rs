use crate::utility::colors::*;
use crate::utility::commands::{get_command_map, get_net_command_map, get_ui_command_map, BypassTarget};
use crate::utility::errors::{HactarError, Result};
use crate::utility::scanning::{select_hactar_port, UartConfig};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serialport::{DataBits, Parity, StopBits};
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct MonitorArgs {
    pub port: Option<String>,
    pub baud: u32,
}

pub fn monitor(args: MonitorArgs) -> Result<()> {
    let uart_config = UartConfig {
        baudrate: args.baud,
        data_bits: DataBits::Eight,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10),
    };

    let port_name = if let Some(port) = args.port {
        port
    } else {
        select_hactar_port(&uart_config)?
    };

    let mut monitor = Monitor::new(&port_name, &uart_config)?;
    monitor.run()
}

struct Monitor {
    port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    running: Arc<Mutex<bool>>,
    rx_thread: Option<thread::JoinHandle<()>>,
}

impl Monitor {
    fn new(port_name: &str, uart_config: &UartConfig) -> Result<Self> {
        let port = serialport::new(port_name, uart_config.baudrate)
            .data_bits(uart_config.data_bits)
            .parity(uart_config.parity)
            .stop_bits(uart_config.stop_bits)
            .timeout(uart_config.timeout)
            .open()?;

        println!("{} {} {}={}", success("Opened port:"), info(port_name), success("baudrate"), info(&format!("{}", uart_config.baudrate)));

        let port = Arc::new(Mutex::new(port));
        let running = Arc::new(Mutex::new(true));

        Ok(Self {
            port,
            running,
            rx_thread: None,
        })
    }

    fn start_reader_thread(&mut self) {
        let port = Arc::clone(&self.port);
        let running = Arc::clone(&self.running);

        let handle = thread::spawn(move || {
            let mut buffer = Vec::new();

            while *running.lock().unwrap() {
                let mut port = port.lock().unwrap();

                // Read available data
                let mut byte = [0u8; 1];
                while port.read(&mut byte).is_ok() {
                    if byte[0] == b'\n' {
                        // Got a complete line
                        if !buffer.is_empty() {
                            if let Ok(line) = String::from_utf8(buffer.clone()) {
                                // Print the line with proper formatting
                                print!("\r\x1b[K{}", line);
                                if !line.ends_with('\n') {
                                    println!();
                                }
                                print!("> ");
                                std::io::stdout().flush().ok();
                            }
                            buffer.clear();
                        }
                    } else {
                        buffer.push(byte[0]);
                    }
                }

                drop(port);
                thread::sleep(Duration::from_millis(50));
            }
        });

        self.rx_thread = Some(handle);
    }

    fn process_command(&self, command: &str) -> Result<()> {
        let command = command.trim();
        if command.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let command_map = get_command_map();

        // Check for simple commands first
        if let Some(cmd_bytes) = command_map.get(command) {
            let mut port = self.port.lock().unwrap();
            port.write_all(cmd_bytes)?;
            return Ok(());
        }

        // Check for bypass commands (ui/net)
        if parts[0] == "ui" || parts[0] == "net" || parts[0] == "loopback" {
            self.process_bypass_command(&parts)?;
            return Ok(());
        }

        println!("Unknown command: {}", command);
        Ok(())
    }

    fn process_bypass_command(&self, parts: &[&str]) -> Result<()> {
        if parts.len() < 2 {
            println!("{} Not enough parameters to determine sub command", error("[ERROR]"));
            return Ok(());
        }

        let target = parts[0];
        let command = parts[1];
        let params = &parts[2..];

        // Get the target type
        let bypass_target = match BypassTarget::from_str(target) {
            Ok(t) => t,
            Err(_) => {
                println!("{} Unknown target: {}", error("[ERROR]"), target);
                return Ok(());
            }
        };

        // Get the command map based on target
        let chip_commands = if target == "ui" {
            get_ui_command_map()
        } else {
            get_net_command_map()
        };

        // Get the command
        let cmd_info = match chip_commands.get(command) {
            Some(c) => c,
            None => {
                println!("{} subcommand {} is unknown", error("[ERROR]"), command);
                return Ok(());
            }
        };

        // Validate number of parameters
        if params.len() < cmd_info.num_params {
            println!(
                "{} Not enough parameters for command {} expected {} got {}",
                error("[ERROR]"),
                command,
                cmd_info.num_params,
                params.len()
            );
            return Ok(());
        }

        if params.len() > cmd_info.num_params {
            println!(
                "{} Too many parameters for command {} expected {} got {}",
                error("[ERROR]"),
                command,
                cmd_info.num_params,
                params.len()
            );
            return Ok(());
        }

        // Build TLV packet
        const HEADER_BYTES: usize = 5; // 1 type + 4 length

        // Calculate lengths
        let mut to_whom_len = HEADER_BYTES;
        let mut command_len = 0;

        for param in params {
            to_whom_len += param.len();
            command_len += param.len();

            if cmd_info.num_params > 1 {
                // Add 4 bytes for each parameter length
                to_whom_len += 4;
                command_len += 4;
            }
        }

        let mut data = Vec::new();

        // MGMT - T (Type)
        data.push(bypass_target as u8);

        // MGMT - L (Length) - little endian
        data.extend_from_slice(&(to_whom_len as u32).to_le_bytes());

        // MGMT - V and UI/NET - T (Command ID)
        data.push(cmd_info.id);

        // UI/NET - L (Length) - little endian
        data.extend_from_slice(&(command_len as u32).to_le_bytes());

        // UI/NET - V (Parameters)
        for param in params {
            if cmd_info.num_params > 1 {
                // Add parameter length
                data.extend_from_slice(&(param.len() as u32).to_le_bytes());
            }
            data.extend_from_slice(param.as_bytes());
        }

        // Send the TLV
        let mut port = self.port.lock().unwrap();
        port.write_all(&data)?;

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        self.start_reader_thread();

        let mut rl = DefaultEditor::new()
            .map_err(|e| HactarError::Other(format!("Failed to create editor: {}", e)))?;

        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.to_lowercase() == "exit" {
                        break;
                    }

                    if !line.is_empty() {
                        rl.add_history_entry(line)
                            .map_err(|e| HactarError::Other(format!("History error: {}", e)))?;
                        self.process_command(line)?;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Stop the reader thread
        *self.running.lock().unwrap() = false;
        if let Some(handle) = self.rx_thread.take() {
            handle.join().ok();
        }

        Ok(())
    }
}
