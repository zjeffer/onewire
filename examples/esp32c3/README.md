# esp-hal esp32c3 example

This example demonstrates how to use the `onewire` crate with a Dallas DS18B20 temperature sensor on an ESP32-C3 using `esp-hal` and `embassy`.

## esp-generate

The skeleton of the example was generated using version `1.3.0` of the `esp-generate` tool:

```bash
cargo install esp-generate
esp-generate --chip=esp32c3 esp32c3 # will launch an interactive prompt to select options
```

As for the options, these are the ones I used:

```bash
--chip esp32c3 -o esp32c3-mini-1 -o unstable-hal -o embassy -o probe-rs -o defmt -o stable-x86_64-unknown-linux-gnu
```

You can change these options as you see fit, depending on your hardware, compiler version preference, extra features, etc.

## onewire example

The example code is mainly in the `read_temperature_sensor` method. It's an async task that continuously reads the temperature from the sensor and logs it. It is inspired by the [embassy_rp2040 example](../embassy_rp2040).

The task is spawned in the `main` function, where we first initialize the peripherals and set up the GPIO pin:

```rust
// set up peripherals
let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);

// configure data pin
let mut data_pin = esp_hal::gpio::Flex::<'static>::new(peripherals.GPIO4);
data_pin.apply_output_config(
	&esp_hal::gpio::OutputConfig::default()
		.with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain),
);
data_pin.set_output_enable(true);
data_pin.set_input_enable(true);
let wire = onewire::OneWire::new(data_pin, false);

// we now have the wire we can spawn the task with:
spawner.spawn(read_temperature_sensor(wire).expect("Failed to spawn temperature sensor task"));
```

## Wiring diagram

Requirements:

- ESP32-C3 or similar
- Dallas DS18B20 temperature sensor
- ~4.7kΩ resistor
- Some wires, breadboard (optional)

The wiring diagram for the example is as follows:

- Connect the ground pin of the ESP32-C3 to the GND pin of the DS18B20.
- Connect the 3.3V pin of the ESP32-C3 to the VCC pin of the DS18B20
- Connect the GPIO4 pin of the ESP32-C3 to the DATA pin of the DS18B20.
- Place a 4.7kΩ resistor between the DATA pin and the VCC pin of the DS18B20 (pull-up resistor).

For a visual representation, https://newbiely.com/tutorials/esp32-c3/esp32-c3-super-mini-temperature-sensor has a nice wiring diagram.

## Running the example

Connect your ESP32 to your computer, then run the cargo command to flash the example:

```bash
cd examples/esp32c3 # ensure you're in this directory
cargo run --release # will build and flash
```

Example output:

```bash
> cargo run --release
   Compiling esp32c3 v0.1.0 (/home/zjeffer/git/embedded-rust/onewire/examples/esp32c3)
    Finished `release` profile [optimized + debuginfo] target(s) in 0.71s
     Running `probe-rs run --chip=esp32c3 --preverify --always-print-stacktrace --no-location target/riscv32imc-unknown-none-elf/release/esp32c3`
      Erasing ✔ 100% [####################] 192.00 KiB @ 673.66 KiB/s (took 0s)
     Finished in 1.40s
[INFO ] Embassy initialized!
[ERROR] No temperature device found # I always get this the first time for some reason, doesn't seem to affect the example
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 27.9375 °C
[INFO ] Temperature: 28.0625 °C # here I started holding the sensor in my hand
[INFO ] Temperature: 28.3125 °C
[INFO ] Temperature: 28.6875 °C
[INFO ] Temperature: 29.0 °C
[INFO ] Temperature: 29.375 °C
[INFO ] Temperature: 29.6875 °C
[INFO ] Temperature: 30.0 °C
[INFO ] Temperature: 30.3125 °C
[INFO ] Temperature: 30.5 °C
^CReceived Ctrl+C, exiting
```
