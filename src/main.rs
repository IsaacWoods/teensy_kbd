#![no_std]
#![no_main]

#![feature(lang_items)]
#![feature(asm)]

mod port;
mod sim;
mod watchdog;

use watchdog::Watchdog;
use sim::{Sim, ClockGate};
use port::{Port, PortName};

extern fn main() -> ! {
    unsafe { Watchdog::new() }.disable();

    let sim = unsafe { Sim::new() };
    let port_c = unsafe { Port::new(PortName::C) };

    sim.enable_clock(ClockGate::PortC);
    let mut led_pin = unsafe { port_c.pin(5) }.make_gpio();

    led_pin.output();
    led_pin.high();

    loop {}
}

extern {
    fn _stack_top() -> !;
}

#[link_section = ".vectors"]
#[no_mangle]
pub static _VECTORS: [unsafe extern fn() -> !; 2] = [
    _stack_top,
    main,
];

#[link_section = ".flashconfig"]
#[no_mangle]
pub static _FLASHCONFIG: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF
];

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    loop { }
}