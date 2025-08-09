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

```
+-----------+                    +-----------+                    +-----------+
|           |                    |           |                    |           |
|           |-------Events------>|           |-------Events------>|           |
|   Board   |                    |    App    |                    |    App    |
|           |<------------------------------------Capabilities----|           |
|           |                    |           |                    |           |
+-----------+                    +-----------+                    +-----------+
                                       ^
                                       |
                                       |
                                     Events
                                       |
                                       |
                                 +-----------+
                                 |           |
                                 |           |
                                 |   ISRs    |
                                 |           |
                                 |           |
                                 +-----------+
```

The main logic of the application is in the function `App::handle(event,
capbilities...)`.
The application takes in events that reflect inputs from the hardware, and acts
on them using the capabilities.  The capabilities are defined using traits,
which are defined in the `app` module and implemented in the `board` module.

In `main.rs`, the board and app are instantiated, as well as a queue of events.
ISRs asynchronously add events to the queue, and the main loop polls for
synchronous events and triggers the app to process events from the queue.

## What it actually does

Right now, the firmware does two things, blink and echo.

The blinking LED demonstrates asynchronous events.  A 1Hz timer is set, which
causes the TIM7 interrupt to fire.  That ISR adds a `Timer1Hz` event to the
queue.  When the app processes that event shortly thereafter, an reference to
the LED is passed as a capability, which the app uses to toggle the LED.

The echo functionality echos text from the UART1 back to UART, with "hello: "
prepended.  This illustrates synchronous events, since the UART receiver has to
be polled.  It also illustrates how the app can use internal state, as it
maintains an internal buffer for characters received from UART.  The send/Tx
side of the UART channel is passed to the app as a capability.

[COTS]: https://en.wikipedia.org/wiki/Commercial_off-the-shelf
[off-the-shelf HAL]: https://docs.rs/stm32f0xx-hal
