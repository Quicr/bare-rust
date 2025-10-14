# Hactar CLI

A command-line tool for flashing firmware and monitoring Hactar devices over serial connections.

## Features

- **Firmware Flashing**: Flash STM32 (mgmt/ui) and ESP32-S3 (net) chips
- **Serial Monitor**: Interactive serial monitor with command completion
- **Auto-Discovery**: Automatically detect Hactar devices on serial ports
- **Multi-Device Support**: Flash multiple devices simultaneously
- **Retry Logic**: Automatic retry on flash failures
- **Verification**: MD5 verification for ESP32 and readback verification for STM32
- **Progress Reporting**: Real-time progress updates during operations

## Installation

### Prerequisites

- Rust toolchain (1.70 or later)
- Serial port drivers for your platform

### Building from Source

```bash
cargo build --release
```

The binary will be available at `target/release/hactar-cli`.

## Usage

### Flash Command

Flash firmware to a Hactar device:

```bash
# Flash a specific chip with auto-detection
hactar-cli flash --chip mgmt --binary firmware.bin

# Flash using a specific port
hactar-cli flash --chip ui --binary firmware.bin --port /dev/ttyUSB0

# Flash with custom baud rate
hactar-cli flash --chip net --binary build/ --baud 460800

# Use external flasher (puts device in flash mode without uploading)
hactar-cli flash --chip net --use-external-flasher
```

#### Supported Chips

- `mgmt` - STM32F072C8T6 management chip
- `ui` - STM32F405RGT6 or STM32F411 user interface chip
- `net` - ESP32-S3 network chip

#### Flash Options

- `--chip <CHIP>` - Target chip (required)
- `--binary <PATH>` - Path to firmware binary or build directory (required unless using external flasher)
- `--port <PORT>` - Serial port (auto-detected if not specified)
- `--baud <RATE>` - Baud rate (default: 230400)
- `--use-external-flasher` - Put device in flash mode without uploading

### Monitor Command

Open an interactive serial monitor:

```bash
# Monitor with auto-detection
hactar-cli monitor

# Monitor specific port
hactar-cli monitor --port /dev/ttyUSB0

# Monitor with custom baud rate
hactar-cli monitor --baud 115200
```

#### Monitor Features

- **Command Completion**: Press Tab to auto-complete available commands
- **Command History**: Use Up/Down arrows to navigate command history
- **Simple Commands**: Direct commands like `who are you`, `version`, etc.
- **Bypass Commands**: Send commands to specific chips:
  - `ui <command> [params]` - Send command to UI chip
  - `net <command> [params]` - Send command to NET chip
- **Exit**: Type `exit`, press Ctrl-C, or Ctrl-D to quit

#### Available Simple Commands

- `who are you` - Identify the device
- `version` - Get firmware version
- `enable logs` - Enable logging output
- `disable logs` - Disable logging output
- `flash mgmt` - Put MGMT chip in flash mode
- `flash ui` - Put UI chip in flash mode
- `flash net` - Put NET chip in flash mode

#### Example Monitor Session

```
> who are you
HELLO, I AM A HACTAR DEVICE

> version
Firmware v1.2.3

> ui set_brightness 128
[UI chip responds]

> exit
```

## Architecture

### Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── config/              # Configuration
│   └── stm32_config.rs  # STM32 chip configurations
├── flasher/             # Firmware flashing
│   ├── flash_impl.rs    # Main flash implementation
│   ├── stm32_uploader.rs# STM32 bootloader protocol
│   ├── esp32_uploader.rs# ESP32 SLIP protocol
│   ├── esp32_slip_packet.rs # SLIP packet encoding
│   └── uart_utils.rs    # Low-level UART utilities
├── monitor/             # Serial monitoring
│   └── monitor_impl.rs  # Interactive monitor
└── utility/             # Shared utilities
    ├── colors.rs        # Terminal color helpers
    ├── commands.rs      # Command definitions
    ├── errors.rs        # Error types
    └── scanning.rs      # Port scanning and detection
```

### Protocols

#### STM32 Bootloader Protocol (AN3155)

The STM32 uploader implements the USART bootloader protocol:

- **Sync**: Establish communication with bootloader
- **Get ID**: Read chip ID
- **Read Memory**: Read from flash memory
- **Write Memory**: Write to flash memory (256 bytes max)
- **Erase Memory**: Extended erase with sector selection
- **Go**: Jump to application code
- **Readback Verification**: Verify written data

#### ESP32-S3 SLIP Protocol

The ESP32 uploader implements the SLIP-based bootloader protocol:

- **SLIP Encoding**: Packet framing with escape sequences
- **Sync**: Synchronize with ROM bootloader
- **SPI Attach/Config**: Configure SPI flash interface
- **Flash Begin/Data/End**: Flash operation sequence
- **MD5 Verification**: Verify flashed firmware integrity

#### TLV Command Protocol

Monitor bypass commands use TLV (Type-Length-Value) encoding:

```
MGMT Layer:
  Type (1 byte): Target chip (15=UI, 16=NET, 17=Loopback)
  Length (4 bytes LE): Total payload length
  Value: Chip command TLV

Chip Layer:
  Type (1 byte): Command ID
  Length (4 bytes LE): Parameter data length
  Value: Parameters (with optional 4-byte lengths for multi-param)
```

## Error Handling

The tool uses comprehensive error handling with descriptive messages:

- `HactarError::SerialPort` - Serial port communication errors
- `HactarError::NoResponse` - Timeout waiting for device response
- `HactarError::Nack` - Device rejected command (NACK)
- `HactarError::SyncFailed` - Failed to sync with bootloader
- `HactarError::VerificationFailed` - Firmware verification failed
- `HactarError::Esp32FlashError` - ESP32 flash operation error
- `HactarError::Md5Mismatch` - MD5 checksum verification failed
- `HactarError::NoDevicesFound` - No Hactar devices detected
- `HactarError::UnsupportedChip` - Invalid chip specified
- `HactarError::ConfigNotFound` - Chip configuration not found

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
# Check compilation
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Adding New Commands

1. Add command definition to `src/utility/commands.rs`
2. Update command maps (`get_ui_command_map` or `get_net_command_map`)
3. Tab completion will automatically include new commands

### Adding New Chip Support

1. Add chip configuration to `src/config/stm32_config.rs` (for STM32)
2. Update chip detection in flasher
3. Test flash and verify operations

## License

Copyright (c) Cisco Systems. All rights reserved.

## Authors

- Brett Regnier <brregnie@cisco.com>

## Conversion from Python

This Rust implementation is a complete rewrite of the original Python hactar-cli tool, maintaining behavioral compatibility while providing:

- Improved performance
- Better error handling
- Type safety
- Cross-platform serial port support
- Modern terminal features (completion, history)
