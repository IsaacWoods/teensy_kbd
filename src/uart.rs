use super::port::Pin;
use bit_field::BitField;
use core::fmt;
use volatile::Volatile;

#[repr(C, packed)]
pub struct Uart {
    bdh: Volatile<u8>,
    bdl: Volatile<u8>,
    c1: Volatile<u8>,
    c2: Volatile<u8>,
    s1: Volatile<u8>,
    s2: Volatile<u8>,
    c3: Volatile<u8>,
    d: Volatile<u8>,
    ma1: Volatile<u8>,
    ma2: Volatile<u8>,
    c4: Volatile<u8>,
    c5: Volatile<u8>,
    ed: Volatile<u8>,
    modem: Volatile<u8>,
    ir: Volatile<u8>,
}

impl Uart {
    pub unsafe fn new(
        id: u8,
        rx: Option<Pin>,
        tx: Option<Pin>,
        clkdiv: (u16, u8),
    ) -> &'static mut Uart {
        let enable_receiver = rx.is_some();
        let enable_transmitter = tx.is_some();

        rx.map(|ref mut rx| rx.set_serial());
        tx.map(|ref mut tx| tx.set_serial());

        let uart = match id {
            0 => &mut *(0x4006_a000 as *mut Uart),
            _ => panic!("Unsupported UART: {}", id),
        };

        uart.c4.update(|c4| {
            c4.set_bits(0..5, clkdiv.1);
        });

        uart.bdh.update(|bdh| {
            bdh.set_bits(0..5, clkdiv.0.get_bits(8..13) as u8);
        });

        uart.bdl.write(clkdiv.0.get_bits(0..8) as u8);

        uart.c2.update(|c2| {
            c2.set_bit(2, enable_receiver);
            c2.set_bit(3, enable_transmitter);
        });

        uart
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            while !self.s1.read().get_bit(7) {}
            self.d.write(byte);
        }

        while !self.s1.read().get_bit(6) {}
        Ok(())
    }
}
