#![no_std]
#![no_main]
#![feature(lang_items, asm, const_fn, decl_macro)]

extern crate bit_field;
extern crate volatile;

mod clock;
mod port;
mod sim;
mod uart;
mod watchdog;

use clock::{Mcg, Oscillator};
use core::fmt::Write;
use core::panic::PanicInfo;
use port::{Port, PortName};
use sim::{ClockGate, Sim};
use uart::Uart;
use watchdog::Watchdog;

static mut UART: Option<&'static mut Uart> = None;

extern "C" fn main() {
    unsafe { Watchdog::new() }.disable();

    let sim = unsafe { Sim::new() };
    let oscillator = unsafe { Oscillator::new() };
    oscillator.enable(clock::TEENSY_32_CAPACITANCE);
    sim.enable_clock_gate(ClockGate::PortB);
    sim.enable_clock_gate(ClockGate::PortC);
    sim.enable_clock_gate(ClockGate::Uart0);

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

    /*
     * Initialise the `Serial1` UART at a baud rate of 9600.
     */
    unsafe {
        let rx = Port::new(PortName::B).pin(16);
        let tx = Port::new(PortName::B).pin(17);
        UART = Some(Uart::new(0, Some(rx), Some(tx), (468, 24)));
    };
    println!("Hello, World!");

    let port_c = unsafe { Port::new(PortName::C) };
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

macro print {
    ($($arg: tt)*) => {
        unsafe {
            match UART {
                Some(ref mut uart) => uart.write_fmt(format_args!($($arg)*)).unwrap(),
                None => panic!("Can't open UART"),
            }
        }
    }
}

macro println {
    ($fmt: expr) => {
        print!(concat!($fmt, "\n\r"));
    },

    ($fmt: expr, $($arg: tt)*) => {
        print!(concat!($fmt, "\n\r"), $($arg)*);
    }
}

#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}
