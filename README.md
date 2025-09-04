Rust on Hactar
==============

This repository contains Rust crates for developing firmware for the Hactar UI
chip. It currently contains three primary crates:

- `ui-app`: The device-independent aspects of the UI chip logic
    * Tests that verify that this logic works as intended, given the right
      inputs.
    * An enum of events that can be emitted by the hardware (basically, async
      inputs)
    * An `Outputs` trait that captures the ways that the app can invoke hardware
      capabilities
- `ui-stm32`: Code to instantiate the app on the Hactar stm32f405 chip
    * An implementation of the traits in the `ui-app` crate based on the Hactar
      EV12 hardware platform.
    * An entry point function that instantiates the app and the board
      abstraction, and routes events from ISRs to the app.
- `ui-laptop`: Code to instantiate the app in a terminal window

## CMOX

The `cmox` directory contains crates that enable the use of the STM
Cryptographic library ("Cortex-M Optimized Crypto Stack").  Following the Rust
"sys-crate" pattern, it contains two crates:

- `cmox-sys`: A bindgen-generated unsafe API and logic to link the CMOX library
- `cmox`: A safe, idiomatic API to the functions exposed by `cmox-sys`

These libraries currently build successfully, including building and running
tests on the UI chip.  However, tests are currently failing due to some
low-level issues.

## Archive

The `archive` directory contains some earlier attempts at getting Rust running
on Hactar, some parts of which may be useful for future development.  See the
README in that directory for more details.

## Quickstart

Prerequisite: [probe-rs]

```
# To run in a terminal window
> cd ui-laptop
> cargo run

# To run on an actual UI chip, via ST-LINK
> cd ui-stm32
# Connect ST-LINK to UI chip
> cargo run
```

The expected behavior is:
* Pushing the PTT (top) button should illuminate the green LED
* Pushing the AI (bottom) button should illuminate the blue LED
* If both buttons are pushed at the same time, the LED should be cyan

## FAQ

**What logic is implemented right now?** Right now, the logic that is
implemented is that the top (PTT) button activates the green LED and the bottom
(AI) button activates the blue LED.

**What external dependencies does the device firmware have?** The primary
external dependencies are:
1. The [stm32f4xx-hal] HAL, which provides a clean, safe interface to the
   hardware with minimal overhead relative to bare register access.
2. The [cortex-m] and [cortex-m-rt] crates, which provide the low-level
   structure for the program.
3. The [heapless] and [panic-halt] crates for some simple utilities.

**What if we want to reduce those dependencies?** We re-write the relevent bits
by hand.  The `heapless` and `panic-halt` dependencies are fairly small and
easily replaced by hand.  The structure provided by the `cortex-m` and
`cortex-m-rt` crates is largely what is done in `startup.rs` in the legacy
`bare-rust` code.  The largest problem would be the HAL -- we could build this
bottom-up (as `bare-rust` attempts) or fork it and strip it down to only what we
need.  I would probably tend toward the latter.

**If we want to change out the stm32f405 for another processor, how do we do
that?** We would create another instantiation crate that would map the new
processor to the needs of the app.  Just like with the current `ui-stm32` and
`ui-laptop` crates.

**How do we accommodate variations between iterations of the device?** The
precise configuration of the EV12 board is almost entirely captured in the file
`ev12.rs`.  The only elements that leak outside that are ISR definitions, which
are short.  To support EV13 and beyond, we can make a parallel `ev13.rs`; its
use and any ISR adaptations should be easy to switch using cargo features.

## TODO

* [ ] UART+DMA connectivity to the MGMT chip
* [ ] UART+DMA connectivity to the NET chip
* [ ] SPI+DMA connectivity to the screen
* [ ] GPIO scanning of the keyboard
* [ ] I2C+I2S connectivity to the audio chip
* [ ] Support for the EV13 board
* [ ] On-device testing using [defmt-test] or [embedded-test] crates
* [ ] Stack measurement using stack painting or SP instrumentation
* [ ] Use [flip-link] to protect against stack overflow
* [ ] More application functionality...

[probe-rs]: https://probe.rs/docs/getting-started/installation/
[stm32f4xx-hal]: https://docs.rs/stm32f4xx-hal/
[cortex-m]: https://docs.rs/cortex-m/
[cortex-m-rt]: https://docs.rs/cortex-m-rt/
[heapless]: https://docs.rs/heapless/
[panic-halt]: https://docs.rs/panic-halt/
[defmt-test]: https://docs.rs/defmt-test/
[embedded-test]: https://docs.rs/embedded-test/
[flip-link]: https://github.com/knurling-rs/flip-link
