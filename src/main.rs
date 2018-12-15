#![no_std]
#![no_main]
#![feature(lang_items, asm, const_fn, decl_macro)]

extern crate bit_field;
extern crate volatile;

mod port;
mod sim;
mod watchdog;
mod clock;

use port::{Port, PortName};
use sim::{ClockGate, Sim};
use watchdog::Watchdog;
use clock::{Oscillator, Mcg};

extern "C" fn main() -> ! {
    unsafe { Watchdog::new() }.disable();

    let port_c = unsafe { Port::new(PortName::C) };

    let sim = unsafe { Sim::new() };
    let oscillator = unsafe { Oscillator::new() };
    oscillator.enable(clock::TEENSY_32_CAPACITANCE);
    sim.enable_clock_gate(ClockGate::PortC);

    /*
     * Set the dividers for the various clocks:
     *      Core: 72Mhz
     *      Bus: 36Mhz
     *      Flash: 24Mhz
     */
    // TODO: set the USB divider
    sim.set_dividers(1, 2, 3);

    // We can now move the MCG to using the external oscillator
    let mcg = unsafe { Mcg::new() };
    mcg.move_to_external_clock();

    let mut led_pin = unsafe { port_c.pin(5) }.make_gpio();
    led_pin.output();
    led_pin.high();

    loop {}
}

extern "C" {
    fn _stack_top() -> !;
}

#[link_section = ".vectors"]
#[no_mangle]
pub static _VECTORS: [unsafe extern "C" fn() -> !; 2] = [_stack_top, main];

#[link_section = ".flashconfig"]
#[no_mangle]
pub static _FLASHCONFIG: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF,
];

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
) -> ! {
    loop {}
}
