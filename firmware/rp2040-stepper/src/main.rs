#![no_std]
#![no_main]

// The macro for our start-up function
use rp_pico::entry;

use cortex_m::prelude::*;

// GPIO traits
use embedded_hal::digital::v2::OutputPin;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Traits for converting integers to amounts of time
use fugit::ExtU32;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp2040_hal as  hal;
use hal::{clocks::init_clocks_and_plls, watchdog::Watchdog, pac };

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    // Note - we don't do any clock set-up in this example. The RP2040 will run
    // at it's default clock speed.

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Our LED output
    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();

        delay.start(500.millis());
        let _ = nb::block!(delay.wait());

        led_pin.set_low().unwrap();

        delay.start(500.millis());
        let _ = nb::block!(delay.wait());
    }
}
