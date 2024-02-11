//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    let mut step = pins.gpio18.into_push_pull_output();

    let mut s1en = pins.gpio19.into_push_pull_output();
    let mut s2en = pins.gpio20.into_push_pull_output();
    let mut s3en = pins.gpio21.into_push_pull_output();
    let mut s4en = pins.gpio22.into_push_pull_output();

    let mut loop_count: u32 = 0;

    s1en.set_high().unwrap();
    s2en.set_high().unwrap();
    s3en.set_high().unwrap();
    s4en.set_low().unwrap();

    loop {
        let delay_us = 800;

        led_pin.toggle().unwrap();

        delay.delay_us(delay_us);
        step.set_high().unwrap();

        delay.delay_us(delay_us);
        step.set_low().unwrap();
    }

    // loop {
    //     led_pin.toggle().unwrap();

    //     let step_us = match (loop_count) {
    //         0..=5 => 10000,
    //         6..=10 => 5000,
    //         11..=15 => 2000,
    //         _ => 1800,
    //     };

    //     info!("loop");

    //     delay.delay_us(step_us);
    //     op4.set_low().unwrap();
    //     op1.set_high().unwrap();

    //     delay.delay_us(step_us);
    //     op1.set_low().unwrap();
    //     op2.set_high().unwrap();

    //     delay.delay_us(step_us);
    //     op2.set_low().unwrap();
    //     op3.set_high().unwrap();

    //     delay.delay_us(step_us);
    //     op3.set_low().unwrap();
    //     op4.set_high().unwrap();

    //     loop_count = loop_count.saturating_add(1);
    // }
}

// End of file
