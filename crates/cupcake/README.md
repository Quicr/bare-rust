Mock-up of Hactar interfaces
============================

This crate is intended to demonstrate the abstraction boundary between the
Hactar application and an interface representing the board.  The `traits` module
contains the core enums and traits that describe the interface, including the
core `Board` trait.  The `ev12` and `ev13` modules instantiate the `Board`
trait, the `task` module defines the application in terms of reactive tasks, and
the `main` module instantiates the application on top of one of the boards.

```
# To run on ev12 (the default)
cargo run

# To run on "ev13" (just to show that we can support multiple boards)
cargo run --no-default-features -F ev13
```

## In reality...

... the various modules here should probably be separate crates:

* A core crate for the traits
* Crates for the various iterations of the hardware
* A crate that defines the application
* A crate that instantiates the application on a given board 

It may be possible to combine these, e.g., putting all of the board-specific
stuff in one crate.

