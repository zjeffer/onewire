#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Flex;
use esp_hal::timer::timg::TimerGroup;

// simply import the onewire crate
use onewire::OneWire;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

///////////// ONEWIRE EXAMPLE /////////////
/// Task to read the Dallas DS18B20 temperature sensor over the 1-wire bus.
#[embassy_executor::task]
pub async fn read_temperature_sensor(mut wire: OneWire<Flex<'static>>) {
    'infinite: loop {
        // reset to test if wire is okay and if any sensor is connected
        if let Err(e) = wire.reset(&mut Delay) {
            error!("Failed to reset 1-wire bus: {:?}", e);
            Timer::after(Duration::from_secs(1)).await;
            continue 'infinite;
        }

        // search for devices on the bus
        let mut search = onewire::DeviceSearch::new();
        let Ok(device) = wire.search_next(&mut search, &mut Delay) else {
            error!("Temperature device search failed");
            continue 'infinite;
        };

        let Some(device) = device else {
            error!("No temperature device found");
            continue 'infinite;
        };

        let sensor = match onewire::ds18b20::DS18B20::new(device) {
            Ok(sensor) => sensor,
            Err(e) => {
                error!("Temperature device is not a DS18B20: {:?}", e);
                continue 'infinite;
            }
        };

        'measure: loop {
            // start measuring temperature
            let resolution = match sensor.measure_temperature(&mut wire, &mut Delay) {
                Ok(resolution) => resolution,
                Err(e) => {
                    error!("Failed to measure temperature: {:?}", e);
                    continue 'measure;
                }
            };

            // wait for measurement to complete
            Timer::after(Duration::from_millis(resolution.time_ms() as u64)).await;

            // get temp
            match sensor.read_temperature(&mut wire, &mut Delay) {
                Ok(temp) => {
                    let (integer, fraction) = onewire::ds18b20::split_temp(temp);
                    let temperature = (integer as f32) + (fraction as f32 / 10000.0);
                    info!("Temperature: {} °C", temperature);
                }
                Err(e) => {
                    error!("Failed to read temperature: {:?}", e);
                    continue 'measure;
                }
            }
        }
    }
}
///////////// END OF ONEWIRE EXAMPLE /////////////

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c3 -o esp32c3-mini-1 -o unstable-hal -o embassy -o probe-rs -o defmt -o stable-x86_64-unknown-linux-gnu

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO2
    // - GPIO8
    // - GPIO9
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO12;
    let _ = peripherals.GPIO13;
    let _ = peripherals.GPIO14;
    let _ = peripherals.GPIO15;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO17;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    ///////////// ONEWIRE EXAMPLE /////////////
    // configure GPIO4 as the data pin for the 1-wire bus
    let mut data_pin = esp_hal::gpio::Flex::<'static>::new(peripherals.GPIO4);
    data_pin.apply_output_config(
        &esp_hal::gpio::OutputConfig::default()
            .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain),
    );
    data_pin.set_output_enable(true);
    data_pin.set_input_enable(true);
    let wire = onewire::OneWire::new(data_pin, false);
    spawner.spawn(read_temperature_sensor(wire).expect("Failed to spawn temperature sensor task"));
    ///////////// END OF ONEWIRE EXAMPLE /////////////

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
