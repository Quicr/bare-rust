FLASHER=../../../hactar/software/flasher/main.py
PORT=/dev/tty.usbserial-10

.PHONY: flash

flash:
	cargo objcopy --release --target=thumbv6m-none-eabi -- -O binary target/mgmt.bin
	python3 ${FLASHER} --baud=115200 --chip=mgmt --port=${PORT} --bin=target/mgmt.bin
