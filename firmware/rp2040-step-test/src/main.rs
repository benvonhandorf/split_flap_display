//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::info;
// use defmt::*;
use defmt_rtt as _;
use embedded_hal::{
    adc::OneShot,
    digital::v2::{OutputPin, ToggleableOutputPin},
};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;
use bsp::hal;

use hal::{
    adc::Adc,
    adc::AdcPin,
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB PicoTool Class Device support
use usbd_picotool_reset::PicoToolReset;

// USB Communications Class Device support
use usbd_serial::SerialPort;

// Used to demonstrate writing formatted strings
use core::{
    fmt::{Error, Write},
    slice::Split,
};
use heapless::String;

use heapless::Vec;
use split_flap_device::split_flap_bit_state::{SensorCalibration, SplitFlapBitState};

const TARGETS: [[char; 4]; 5] = [
    ['B', 'V', 'H', ' '],
    ['A', 'L', 'L', 'T'],
    ['R', 'A', 'I', 'L'],
    ['S', ' ', ' ', ' '],
    [0x01, 0x02, 0x03, 0x04],
];

#[entry]
fn main() -> ! {
    // info!("Program start");
    let mut peripherals = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
    let sio = Sio::new(peripherals.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        peripherals.USBCTRL_REGS,
        peripherals.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut peripherals.RESETS,
    ));

    // // Set up the USB PicoTool Class Device driver
    let mut picotool: PicoToolReset<_> = PicoToolReset::new(&usb_bus);

    // // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    let pins = bsp::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let mut adc = Adc::new(peripherals.ADC, &mut peripherals.RESETS);

    // Configure one of the pins as an ADC input
    let mut sensor_pin_0 = AdcPin::new(pins.gpio27.into_floating_input());
    let mut sensor_pin_1 = AdcPin::new(pins.gpio27.into_floating_input());
    let mut sensor_pin_2 = AdcPin::new(pins.gpio27.into_floating_input());
    let mut sensor_pin_3 = AdcPin::new(pins.gpio27.into_floating_input());

    let mut led_pin = pins.led.into_push_pull_output();

    let mut step = pins.gpio18.into_push_pull_output();

    let mut s0en = pins.gpio19.into_push_pull_output();
    let mut s1en = pins.gpio20.into_push_pull_output();
    let mut s2en = pins.gpio21.into_push_pull_output();
    let mut s3en = pins.gpio22.into_push_pull_output();

    let mut sensor_0: u64 = 0;
    let mut sensor_1: u64 = 0;
    let mut sensor_2: u64 = 0;
    let mut sensor_3: u64 = 0;

    s0en.set_low().unwrap();
    s1en.set_low().unwrap();
    s2en.set_low().unwrap();
    s3en.set_low().unwrap();

    const STEP_DELAY_US: u32 = 900;
    const STEP_DELAY_TARGET_MS: u32 = 1000;

    const STEPS_PER_FLAP: u32 = 58;

    const HOME_OFFSET: u32 = STEPS_PER_FLAP * 5;

    let mut target_idx = 0;

    let sensor_calibration = SensorCalibration {
        trigger_value: 2200,
        untrigger_value: 2100,
    };

    let mut BITS: [SplitFlapBitState; 4] = [
        SplitFlapBitState::new(sensor_calibration, STEPS_PER_FLAP, HOME_OFFSET),
        SplitFlapBitState::new(sensor_calibration, STEPS_PER_FLAP, HOME_OFFSET),
        SplitFlapBitState::new(sensor_calibration, STEPS_PER_FLAP, HOME_OFFSET),
        SplitFlapBitState::new(sensor_calibration, STEPS_PER_FLAP, HOME_OFFSET),
    ];

    loop {
        sensor_0 = adc.read(&mut sensor_pin_0).unwrap();
        sensor_1 = adc.read(&mut sensor_pin_1).unwrap();
        sensor_2 = adc.read(&mut sensor_pin_2).unwrap();
        sensor_3 = adc.read(&mut sensor_pin_3).unwrap();

        let s0_process = BITS[0].process(sensor_0 as u32);
        let s1_process = BITS[1].process(sensor_1 as u32);
        let s2_process = BITS[1].process(sensor_2 as u32);
        let s3_process = BITS[1].process(sensor_3 as u32);

        if s0_process {
            s0en.set_high().unwrap();
        } else {
            s0en.set_low().unwrap();
        }

        if s1_process {
            s1en.set_high().unwrap();
        } else {
            s1en.set_low().unwrap();
        }

        if s2_process {
            s2en.set_high().unwrap();
        } else {
            s2en.set_low().unwrap();
        }

        if s3_process {
            s3en.set_high().unwrap();
        } else {
            s3en.set_low().unwrap();
        }

        if !s0_process && !s1_process && !s2_process && !s3_process {
            //If we've stopped stepping, we can delay briefly and then advance to the next character
            //to be displayed

            delay.delay_ms(STEP_DELAY_TARGET_MS);

            target_idx = (target_idx + 1) % TARGETS.len();
            let targets = TARGETS[target_idx];

            for idx in 0..4 {
                BITS[idx].set_target_character(targets[idx] as u8);
            }

            info!("New targets: {}", targets);
        }

        delay.delay_us(STEP_DELAY_US);
        step.set_high().unwrap();

        delay.delay_us(STEP_DELAY_US);
        step.set_low().unwrap();

        // info!("ADC: {}", reading);

        // let _ = serial.write(b"Loop\r\n");

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    // Convert to upper case
                    buf.iter_mut().take(count).for_each(|b| {
                        b.make_ascii_uppercase();
                    });
                    // // Send back to the host
                    // let mut wr_ptr = &buf[..count];
                    // while !wr_ptr.is_empty() {
                    //     match serial.write(wr_ptr) {
                    //         Ok(len) => wr_ptr = &wr_ptr[len..],
                    //         // On error, just drop unwritten data.
                    //         // One possible error is Err(WouldBlock), meaning the USB
                    //         // write buffer is full.
                    //         Err(_) => break,
                    //     };
                    // }
                }
            }
        }
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
