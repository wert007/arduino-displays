rub: build
    cargo run --quiet

build:
    @cargo build --quiet
    @cargo avr-build --elf-file "target/avr-atmega328p/debug/er-epm0154-2b.elf" --error
