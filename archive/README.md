Hactar-rs Archive
=================

This directory is a holding area for old code from previous attempts to get Rust
going on the Hactar device.  This file attempts to highlight what might be
useful for future work about each earlier attempt.

`bare-rust` - This is Cullen's original approach, writing everything from hand.
Most useful for showing how to accomplish low-level things with no dependencies.

`neo` - This is Richard's translation and slight modernization of Cullen's
approach.  Most useful part is probably the `svdgen` tool and its integration
with `build.rs`.  `svdgen` provides similar functionality to `svd2rust`, but
without introducing additional dependencies.

`mgmt-embassy` - This is a small experiment in an Embassy-based firmware for the
MGMT chip, showing how to do UART with DMA.  It includes a generic `pipe`
function to pipe a UART Rx to a UART Tx, which is basically what the MGMT chip
does.  Might be useful if we ever decide that it would be easier to program the
MGMT chip in Rust.

`net-idf` - This is a small experiment in using the ESP-provided Rust stack to
build a firmware for the NET chip.  It boots the chip, connects to wifi, and
makes a connection to a WebSocket server.  Might be useful for making
special-purpose NET firmware for development purposes.
