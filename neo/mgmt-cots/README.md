MGMT with [COTS] parts
======================

In the interest of getting the firmware design moving quickly, this crate is an
effort to get the management chip running using an [off-the-shelf HAL] as much as
possible.  With the idea that we can strip it down to something more minimal
later, once we have things working.

## Architecture

In brief:

* The `board` module defines how the hardware board is put together.
* The `app` module defines the application logic and the traits of the board
  that it depends on.
* The `main` file instantiates the board and the app, and organizes the
  execution in the main function and ISRs.


[COTS]: https://en.wikipedia.org/wiki/Commercial_off-the-shelf
[off-the-shelf HAL]: https://docs.rs/stm32f0xx-hal
