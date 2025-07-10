
.PHONY: build flash build-mgmt run-mgmt test doc clean cov all

all: build build-mgmt test doc 

build:
	cd ui && cargo build

flash:
	cd ui && cargo flash --release --chip STM32F405RGTx

build-mgmt:
	cd mgmt && cargo build

flash-mgmt:
	cd mgmt && cargo flash --release --chip STM32F072CBTx

run-mgmt:
	echo run "openocd -f mgmt/openocd.cfg" in background
	echo run something like "screen /dev/tty.usbserial-120 115200" in another window
	echo do "cd mgmt; cargo run --bin mgmt --features hal/stm32f072 --target=thumbv6m-none-eabi  --verbose"

# XXX(RLB): UI tests are currently not run, because the `ui` crate doesn't
# 					allow switching architectures.
test:
	cd hal && cargo test -F stm32f405,std
	cd bsp && cargo test -F board-sim,hal/stm32f405,hal/std

# XXX(RLB): This will not work right now
doc:
	cargo doc --workspace --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit

clean:
	cd hal && cargo clean
	cd bsp && cargo clean
	cd mgmt && cargo clean
	cd ui && cargo clean

# XXX(RLB): This will not work right now
cov:
	cargo llvm-cov --workspace --lib --tests --bin ui --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1
