New Architecture
================

This directory contains a prototype of a new structure for Rust code on the
Hactar UI chip.  It currently contains three crates:

- `ui-app`
    * The device-independent aspects of the UI chip logic
    * Tests that verify that this logic works as intended, given the right
      inputs.
    * An enum of events that can be emitted by the hardware (basically, async
      inputs)
    * An `Outputs` trait that captures the ways that the app can invoke hardware
      capabilities
- `ui-stm32`
    * Code to instantiate the app on the Hactar stm32f405 chip
    * An implementation of the traits in the `ui-app` crate based on the Hactar
      EV12 hardware platform.
    * An entry point function that instantiates the app and the board
      abstraction, and routes events from ISRs to the app.
- `ui-laptop`
    * Code to instantiate the app in a terminal window

## Quickstart

```
# To run in a terminal window
> cd ui-laptop
> cargo run

# To run on an EV12 board
> cd ui-stm32
> cargo build
# Connect USB-C cable to Hactar
# You may have to modify the Makefile to point to a local Hactar flasher script
> make flash
```

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

## Known Gaps vis à vis `bare-rust`

* Stack measurement - should be possible to re-add
* UART support - should be able to add via off-the-shelf HAL 

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

[stm32f4xx-hal]: https://docs.rs/stm32f4xx-hal/
[cortex-m]: https://docs.rs/cortex-m/
[cortex-m-rt]: https://docs.rs/cortex-m-rt/
[heapless]: https://docs.rs/heapless/
[panic-halt]: https://docs.rs/panic-halt/
[defmt-test]: https://docs.rs/defmt-test/
[embedded-test]: https://docs.rs/embedded-test/
[flip-link]: https://github.com/knurling-rs/flip-link
