# MGMT Firmware - Architecture Summary

## Overview

This is the management firmware for a HACTAR device running on an STM32F072CB microcontroller. The firmware acts as a routing and control layer between three UART interfaces:

- **USB UART** (USART1): Connected to a host computer
- **UI UART** (USART2): Connected to a UI chip (STM32-based)
- **NET UART** (USART3): Connected to a network chip (ESP-based)

The primary functions are:
1. **UART Routing**: Dynamically route data between the three UARTs
2. **Command Processing**: Execute commands received from USB for device control
3. **Chip Management**: Control power and boot modes of UI and NET chips
4. **Flash Mode**: Enable firmware updates for UI and NET chips via USB

## Rust Concepts for C/C++ Programmers

If you're coming from C/C++ embedded development, here are the key Rust concepts used in this firmware:

### Ownership and Borrowing

Rust has a unique ownership system that eliminates the need for manual memory management:

- **Ownership**: Every value has a single owner. When the owner goes out of scope, the value is automatically cleaned up.
- **Borrowing**: You can temporarily borrow a reference to a value without taking ownership
  - `&T` - immutable reference (like `const T*` in C++)
  - `&mut T` - mutable reference (like `T*` in C++, but exclusive)

**Key Rule**: You can have either ONE mutable reference OR multiple immutable references, but never both at the same time. This prevents data races at compile time.

Example from the code:
```rust
pub async fn route_data<'a>(&mut self, src: Interface, buf: &'a mut [u8])
```
- `&mut self` - we're borrowing the State mutably
- `buf: &'a mut [u8]` - we're borrowing a mutable slice with lifetime `'a`

### Lifetimes

Lifetimes are Rust's way of tracking how long references are valid. They prevent dangling pointers at compile time.

**Syntax**: `'a`, `'d`, `'static` - these are lifetime parameters

In C, you might do:
```c
char* get_name() {
    char name[10] = "hello";
    return name;  // BUG: returns pointer to local variable!
}
```

Rust prevents this at compile time with lifetimes.

**In this firmware**:
```rust
pub struct State<'d> {
    pub usb_rx: RingBufferedUartRx<'d>,
    // ...
}
```

The `'d` lifetime says: "The State struct contains references that are valid for lifetime `'d`". These references are to the DMA buffers passed to `State::new()`:

```rust
pub fn new(
    usb_rx_buf: &'d mut [u8],  // This buffer must live as long as State
    ui_rx_buf: &'d mut [u8],
    net_rx_buf: &'d mut [u8],
) -> Self
```

The compiler ensures the buffers aren't deallocated while State is still using them.

**Special lifetime**: `'static` means "lives for the entire program". In embedded systems, this is often used for peripherals and global resources that never go away.

### Async/Await

In C embedded programming, you typically use:
- **Interrupt Service Routines (ISRs)** for hardware events
- **Callbacks** registered with peripheral drivers
- **State machines** for complex operations

Rust's async/await provides a cleaner alternative:

```rust
async fn handle_command(&mut self, buf: &mut [u8]) {
    self.usb_rx.read_exact(&mut type_len).await.unwrap();
    // ... more code ...
}
```

**What `async` means**:
- The function can be suspended and resumed
- When you call `.await`, the function yields control back to the executor
- Other async tasks can run while this one is waiting

**How it works** (simplified):
1. Embassy executor runs on top of interrupts
2. When UART data arrives, interrupt fires
3. Interrupt wakes up the async task waiting for that data
4. Task resumes from its `.await` point

**Benefits over ISRs**:
- Write sequential code instead of state machines
- No manual state management
- No callback hell
- Memory-safe: compiler ensures no data races

### Match Expressions

Similar to `switch` in C, but much more powerful:

```rust
match command {
    Command::Version => self.usb_tx.write(VERSION).await.unwrap(),
    Command::Reset => {
        self.reset_ui().await;
        self.reset_net().await;
    }
    _ => unreachable!("Invalid command"),
}
```

- Must be exhaustive (all cases covered)
- Can match patterns, not just constants
- Can extract data from enums

### Enums

Rust enums are more powerful than C enums - they can carry data:

```rust
pub enum Interface {
    Drop,      // Discard data
    Usb,       // Route to USB
    Ui,        // Route to UI
    Net,       // Route to NET
    Command,   // Process as command
}
```

This is a simple enum (like C). But Rust enums can also be:

```rust
enum Result<T, E> {
    Ok(T),      // Success with value of type T
    Err(E),     // Error with value of type E
}
```

### Traits

Traits are like interfaces in other languages. They define behavior:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface { ... }
```

The `#[derive(...)]` automatically implements these traits:
- `Debug` - can be printed for debugging
- `Clone` - can be copied explicitly
- `Copy` - can be copied implicitly (like C integers)
- `PartialEq, Eq` - can be compared with `==`

### No Runtime, No Heap

Like C embedded code, this firmware:
- Uses `#![no_std]` - no standard library
- Uses `#![no_main]` - no normal main function
- No heap allocation - everything is stack or static
- No dynamic dispatch (virtual functions) unless explicit

## Module Structure

```
src/
├── main.rs       - Entry point and main loop
├── state.rs      - Core state machine and command handling
├── drivers.rs    - Hardware drivers (GPIO, LED control)
└── commands.rs   - Command definitions and constants
```

### main.rs

The entry point. Very simple:
1. Allocate DMA buffers on the stack
2. Create State instance
3. Run main loop forever:
   - Blink LED
   - Handle USB (either commands or routing)
   - Route UI data
   - Route NET data
   - Sleep 100μs

### state.rs

Contains the core State struct and all command handling logic. This is the "business logic" of the firmware.

### drivers.rs

Low-level hardware drivers for:
- RGB LEDs (active-low)
- UI chip control (STM32 with boot0/boot1 pins)
- NET chip control (ESP with boot pin)

### commands.rs

Just definitions - the Command enum and response constants.

## Major Types

### Command Enum

Defines all supported commands in the TLV protocol:

```rust
#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum Command {
    Version = 0,
    WhoAreYou = 1,
    HardReset = 2,
    Reset = 3,
    // ... 18 commands total
}
```

**Key attributes**:
- `#[repr(u8)]` - store as a single byte (like C enum)
- `TryFromPrimitive` - can convert from `u8`, returns error if invalid

Commands fall into three categories:

1. **Information**: Version, WhoAreYou
2. **Control**: Reset variants, Flash mode, logging control
3. **Data forwarding**: ToUi, ToNet, ToUsb (with payload)

### Interface Enum

Represents routing destinations:

```rust
pub enum Interface {
    Drop,      // Discard data (like /dev/null)
    Usb,       // Send to USB
    Ui,        // Send to UI
    Net,       // Send to NET
    Command,   // Parse as command
}
```

Used in `UartRouting` to configure where each UART's received data goes.

### State Struct

The main state container - owns all hardware resources:

```rust
pub struct State<'d> {
    // LEDs
    pub led_a: RgbLed,
    pub led_b: RgbLed,

    // Chip control
    pub ui_control: UiControl,
    pub net_control: NetControl,

    // UART TX/RX (split for independent operation)
    pub usb_tx: UartTx<'static, Async>,
    pub usb_rx: RingBufferedUartRx<'d>,

    pub ui_tx: UartTx<'static, Async>,
    pub ui_rx: RingBufferedUartRx<'d>,

    pub net_tx: UartTx<'static, Async>,
    pub net_rx: RingBufferedUartRx<'d>,

    // Routing configuration
    pub routing: UartRouting,
}
```

**Lifetime `'d`**:
- Applied to RX ring buffers
- These buffers are passed in from main.rs
- They live on main's stack for the entire program
- The `'d` lifetime ensures State doesn't outlive those buffers

**Key Methods**:

#### `State::new()`
Initializes all hardware:
- Calls `embassy_stm32::init()` to configure peripherals
- Creates LED drivers
- Creates chip control drivers
- Configures all three UARTs with DMA
- Sets default routing (USB→Command, UI→USB, NET→USB)

#### `route_data(src: Interface, buf: &mut [u8])`
Generic data routing:
1. Look up destination from routing table
2. Read from source UART into buffer
3. Write buffer to destination UART

#### `handle_command(buf: &mut [u8])`
Command processing:
1. Read 5-byte header: `[type: u8][length: u32 BE]`
2. Parse command type
3. If length is 0: execute direct command
4. If length > 0: execute forwarding command with data

#### Direct Commands
Commands with no payload - execute immediately and send response:
- Version, WhoAreYou: return info
- Reset variants: reset chips
- Flash mode: reconfigure for bootloader
- Log control: update routing

#### Forwarding Commands
Commands with payload data - stream data chunk by chunk:
- ToUi/ToNet/ToUsb: read data from USB and forward to destination
- Can handle large transfers (chunks of 64 bytes)

### Hardware Drivers

#### RgbLed
Controls a common-cathode RGB LED:
- `set_rgb(r, g, b)` - set color
- `toggle_red/green/blue()` - toggle individual channels
- Uses active-low logic (output LOW = LED ON)

#### UiControl
Controls an STM32-based UI chip:
- `nrst` - reset pin (Flex GPIO - can be input or output)
- `boot0, boot1` - boot mode selection pins

**Methods**:
- `normal_mode()` - boot from flash (boot0=0, boot1=1)
- `bootloader_mode()` - boot from system memory (boot0=1, boot1=0)
- `hold_in_reset()` - keep chip in reset state
- `power_cycle()` - reset pulse sequence

The reset pin uses a special sequence:
1. Set as output, drive low (reset asserted)
2. Delay 10ms
3. Drive high, delay 10ms
4. Drive low, delay 10ms
5. Set as input with pull-up (release control to external circuit)

This matches the original C implementation's behavior.

#### NetControl
Controls an ESP-based NET chip:
- `nrst` - reset pin
- `boot` - boot mode selection (high=normal, low=bootloader)

**Methods**:
- `normal_mode()` - boot from flash
- `bootloader_mode()` - boot from ROM bootloader
- `hold_in_reset()` - keep chip in reset state

Simpler than UI because ESP only has one boot pin.

## Program Flow

### Startup

1. **main.rs**: Allocate DMA buffers on stack (1024 bytes each)
2. **State::new()**:
   - Initialize Embassy runtime
   - Configure GPIO for LEDs (turn off)
   - Configure GPIO for chip control
   - Initialize all three UARTs:
     - 115200 baud, 8N1
     - DMA for TX and RX
     - Ring buffer for RX (efficient async reading)
   - Set default routing:
     - USB → Command (parse TLV commands)
     - UI → USB (forward UI logs to host)
     - NET → USB (forward NET logs to host)
3. **Main loop**: Start processing

### Main Loop

Each iteration (every 100μs):

```rust
loop {
    // Visual heartbeat
    if led_timer.elapsed().as_millis() >= 1000 {
        s.led_a.toggle_green();
        led_timer = embassy_time::Instant::now();
    }

    // USB: special handling (command vs routing)
    if s.routing.usb == Interface::Command {
        s.handle_command(&mut buf).await;
    } else {
        s.route_data(Interface::Usb, &mut buf).await;
    }

    // UI and NET: always just routing
    s.route_data(Interface::Ui, &mut buf).await;
    s.route_data(Interface::Net, &mut buf).await;

    embassy_time::Timer::after(Duration::from_micros(100)).await;
}
```

**Why only USB is special**:
- UI and NET can only route data
- USB can either:
  - Route data (when in flash mode)
  - Parse commands (normal mode)

### Data Routing

When `route_data()` is called:

1. **Look up destination**:
   ```rust
   let dst = match src {
       Interface::Usb => self.routing.usb,
       Interface::Ui => self.routing.ui,
       Interface::Net => self.routing.net,
       _ => unreachable!(),
   };
   ```

2. **Read from source** (async, waits for data):
   ```rust
   let n = self.usb_rx.read(buf).await.unwrap();
   ```

   This `.await` suspends the task until data arrives. The Embassy executor handles the UART interrupt and wakes the task.

3. **Write to destination** (async, waits for TX complete):
   ```rust
   match dst {
       Interface::Usb => self.usb_tx.write(data).await.unwrap(),
       Interface::Ui => self.ui_tx.write(data).await.unwrap(),
       Interface::Net => self.net_tx.write(data).await.unwrap(),
       Interface::Drop => {}, // discard
       Interface::Command => {}, // handled separately
   }
   ```

### Command Processing

The command protocol is a simple TLV (Type-Length-Value) format:

```
┌────────┬────────────────┬──────────────┐
│ Type   │ Length         │ Value        │
│ 1 byte │ 4 bytes (BE)   │ N bytes      │
└────────┴────────────────┴──────────────┘
```

**Note**: Length is **big-endian** (network byte order).

#### Command Flow

1. **Read header** (blocking read, waits for exactly 5 bytes):
   ```rust
   let mut type_len = [0u8; 5];
   self.usb_rx.read_exact(&mut type_len).await.unwrap();
   ```

2. **Parse command type**:
   ```rust
   let Ok(command) = Command::try_from(type_len[0]) else {
       defmt::warn!("Invalid command: {}", type_len[0]);
       return;
   };
   ```

3. **Parse length**:
   ```rust
   let mut len_bytes = [0u8; 4];
   len_bytes.copy_from_slice(&type_len[1..]);
   let len = u32::from_be_bytes(len_bytes) as usize;
   ```

4. **Execute command**:
   - If `len == 0`: Direct command (no payload)
   - If `len > 0`: Forwarding command (with payload)

#### Direct Command Example: Version

```rust
Command::Version => self.usb_tx.write(VERSION).await.unwrap(),
```

Simply writes the version string back to USB.

#### Direct Command Example: FlashUi

```rust
Command::FlashUi => {
    info!("Entering UI flash mode");

    // Hold NET in reset (so it doesn't interfere)
    self.net_control.hold_in_reset();

    // Configure routing: USB ↔ UI (bidirectional)
    self.routing.usb = Interface::Ui;
    self.routing.ui = Interface::Usb;
    self.routing.net = Interface::Drop;

    // Send OK byte to host
    self.usb_tx.write(&[OK_BYTE]).await;

    // Reconfigure USB UART to 9E1 (for STM32 bootloader)
    let config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits9;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityEven;
        config
    };

    self.usb_rx.set_config(&config).unwrap();
    self.usb_tx.set_config(&config).unwrap();

    // Wait for UART reconfiguration
    embassy_time::Timer::after(Duration::from_millis(200)).await;

    // Put UI chip into bootloader mode
    self.ui_control.bootloader_mode();

    // Send Ready byte to host
    self.usb_tx.write(&[READY_BYTE]).await;
}
```

**Flash Mode Sequence**:
1. Isolate the chip being flashed (hold other chip in reset)
2. Configure routing for transparent passthrough
3. Acknowledge to host (OK_BYTE = 0x80)
4. Reconfigure UART if needed (STM32 bootloader uses 9E1)
5. Put chip into bootloader mode
6. Signal ready to host (READY_BYTE = 0x81)

Now the host can talk directly to the chip's bootloader through the management controller.

#### Forwarding Command Example: ToUi

```rust
async fn forwarding_command(&mut self, command: Command, len: usize, buf: &mut [u8]) {
    let mut remaining = len;
    while remaining != 0 {
        // Read chunk (up to buffer size)
        let curr_len = buf.len().min(remaining);
        let curr = &mut buf[..curr_len];

        self.usb_rx.read_exact(curr).await.unwrap();

        // Forward to destination
        match command {
            Command::ToUi => {
                self.led_b.toggle_blue();
                self.ui_tx.write(buf).await.unwrap();
            }
            Command::ToNet => {
                self.led_b.toggle_green();
                self.net_tx.write(buf).await.unwrap();
            }
            Command::ToUsb => {
                self.led_b.toggle_red();
                self.usb_tx.write(buf).await.unwrap();
            }
            _ => unreachable!(),
        }

        remaining -= curr_len;
    }
}
```

**Chunked transfer**:
- Buffer is only 64 bytes
- Payload might be megabytes (firmware update)
- Read and forward in chunks
- LED toggles on each chunk (visual feedback)

## Async/Await Execution Model

For C programmers, the async model might seem magical. Here's what's really happening:

### Traditional C Approach

```c
// UART interrupt handler
void USART1_IRQHandler(void) {
    if (USART1->ISR & USART_ISR_RXNE) {
        uint8_t byte = USART1->RDR;
        ring_buffer_push(&rx_buffer, byte);
        // Set flag or call callback
        data_available = true;
    }
}

// Main loop
while (1) {
    if (data_available) {
        process_data();
        data_available = false;
    }
}
```

Challenges:
- Manual state management
- Callbacks or flags
- Data races if not careful
- Complex for multi-step operations

### Rust Async Approach

```rust
// No manual ISR - Embassy handles it
async fn handle_data() {
    // This looks synchronous, but yields on .await
    let data = uart.read(&mut buf).await.unwrap();
    process_data(data);
}
```

**What happens**:

1. **Task starts**: `uart.read()` initiates a read
2. **No data yet**: `.await` suspends the task (yields to executor)
3. **Interrupt fires**: UART has data
4. **Executor wakes task**: Control returns to the task
5. **Task resumes**: Continue after `.await`

**Memory**: The task's state (local variables, position in code) is stored in a "Future" object on the stack.

**Benefits**:
- Sequential code (easier to read)
- No manual state machines
- Compiler prevents data races
- Zero-cost abstraction (compiled to state machine)

### Why It's Safe

In C, you might have:
```c
// BAD: Race condition
uint8_t buffer[64];

void ISR() {
    // Writes to buffer
}

void main_loop() {
    // Reads from buffer - could conflict!
}
```

In Rust:
```rust
async fn handle_data(&mut self, buf: &mut [u8]) {
    self.uart.read(buf).await.unwrap();
    // Compiler ensures ISR can't access buf while we're using it
}
```

The `&mut` reference is **exclusive**. The compiler guarantees no other code can access the buffer simultaneously.

## Key Design Decisions

### Single State Struct

All hardware resources are owned by one `State` instance:
- **Pro**: Clear ownership, no global variables
- **Pro**: Easy to pass around, easy to test
- **Con**: Large struct (but stack-allocated)

### Async Single-Task

Unlike the original multi-task approach, this uses a single async task:
- **Pro**: Lower stack usage
- **Pro**: Simpler to reason about
- **Pro**: No need for synchronization primitives
- **Con**: Sequential processing (but fast enough for this application)

### Ring-Buffered UART

Uses DMA ring buffers for reception:
- **Pro**: Efficient (no byte-by-byte copying)
- **Pro**: Don't lose data if processing is slow
- **Pro**: Async-friendly (wake when data available)

### No Interior Mutability

Original versions used `Mutex` for shared access. Refactored to use direct `&mut` references:
- **Pro**: Simpler code
- **Pro**: No runtime overhead
- **Pro**: More explicit borrowing
- **Con**: Can't share references (but we don't need to)

## Memory Layout

All memory is static or stack-allocated:

```
Stack (main function):
  ├── usb_rx_buf: [u8; 1024]   // DMA buffer
  ├── ui_rx_buf: [u8; 1024]    // DMA buffer
  ├── net_rx_buf: [u8; 1024]   // DMA buffer
  ├── s: State<'d>             // Owns all resources
  ├── buf: [u8; 64]            // Temp buffer for commands
  └── led_timer: Instant       // Timestamp

State struct contains:
  ├── LED drivers              // GPIO outputs
  ├── Chip control drivers     // GPIO outputs
  ├── UART TX (3x)            // Async writers
  ├── UART RX (3x)            // Ring-buffered readers (reference DMA buffers)
  └── routing: UartRouting    // Small struct (3 bytes)
```

**Total RAM usage**: ~3KB for DMA buffers + ~100 bytes for State + stack overhead

Compare to C:
- No malloc/free
- No heap fragmentation
- Deterministic memory usage
- No memory leaks possible

## Error Handling

Uses Rust's `Result<T, E>` type:

```rust
let Ok(command) = Command::try_from(type_len[0]) else {
    defmt::warn!("Invalid command: {}", type_len[0]);
    return;
};
```

This is a "let-else" pattern:
- Try to parse command from byte
- If it fails, log warning and return
- If it succeeds, continue with `command` variable

Many places use `.unwrap()`:
```rust
self.usb_rx.read_exact(&mut type_len).await.unwrap();
```

**Why unwrap?**:
- UART operations shouldn't fail in normal operation
- If they do, it's a critical error
- `.unwrap()` panics (halts the system) on error
- In production, you might handle errors more gracefully

## Debugging

Uses the `defmt` logging framework:

```rust
info!("Starting main loop");
defmt::warn!("Invalid command: {}", type_len[0]);
```

**defmt** is a zero-cost logging framework:
- Logs are formatted on the host, not the device
- Only sends format string index + data
- Very low overhead
- Transmitted via RTT (Real-Time Transfer) - a debug channel

To see logs:
```bash
probe-rs run --chip STM32F072CBTx
```

## Comparison to Original C Code

| Aspect | C Version | Rust Version |
|--------|-----------|--------------|
| **ISRs** | Manual ISR handlers | Embassy handles interrupts |
| **State** | Global variables | Single State struct |
| **Buffers** | `static uint8_t buf[1024]` | Stack-allocated, lifetime-checked |
| **Callbacks** | Function pointers | Async/await |
| **Errors** | Return codes, easy to ignore | `Result<T, E>`, hard to ignore |
| **Concurrency** | Volatile, critical sections | Compiler-enforced safety |
| **Memory Safety** | Manual bounds checking | Automatic bounds checking |
| **Null Pointers** | Possible | Impossible (uses `Option<T>`) |

## Further Reading

To understand this codebase better:

1. **Embassy Framework**: https://embassy.dev/
   - Async executor for embedded Rust
   - HAL for STM32 and other MCUs

2. **The Rust Book**: https://doc.rust-lang.org/book/
   - Chapter 4: Ownership
   - Chapter 10: Lifetimes
   - Chapter 6: Enums and Pattern Matching

3. **Embedded Rust Book**: https://rust-embedded.github.io/book/
   - Practical guide to embedded Rust
   - Explains `#![no_std]` environment

4. **Async Rust**: https://rust-lang.github.io/async-book/
   - How async/await works
   - Futures and executors

## Summary

This firmware demonstrates modern embedded Rust:
- **Memory safety** without runtime overhead
- **Async/await** instead of manual ISRs and callbacks
- **Strong typing** prevents entire classes of bugs
- **Zero-cost abstractions** compile to efficient machine code

For a C programmer, the learning curve is:
1. Understand ownership and borrowing (hardest part)
2. Understand lifetimes (builds on #1)
3. Understand async/await (different from ISRs, but logical)
4. Appreciate the benefits (safety + performance)

The code is more verbose than C, but the compiler catches many bugs at compile time that would be runtime bugs in C. Once it compiles, it's very likely to work correctly.
