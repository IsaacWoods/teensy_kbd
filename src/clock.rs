use bit_field::BitField;
use volatile::Volatile;

/// The crystal oscillator on the Teensy 3.2 has a capacitance of 10pf
pub const TEENSY_32_CAPACITANCE: u8 = 10;

#[repr(C, packed)]
pub struct Oscillator {
    control_reg: Volatile<u8>,
}

impl Oscillator {
    pub unsafe fn new() -> &'static mut Oscillator {
        &mut *(0x4006_5000 as *mut Oscillator)
    }

    pub fn enable(&mut self, capacitance: u8) {
        let mut control_reg: u8 = 0;

        // The capacitance bits are backwards and start as 2pf, so we swizzle them here
        control_reg.set_bit(3, capacitance.get_bit(1));
        control_reg.set_bit(2, capacitance.get_bit(2));
        control_reg.set_bit(1, capacitance.get_bit(3));
        control_reg.set_bit(0, capacitance.get_bit(4));

        // We can then make the crystal oscillator do the wiggling
        control_reg.set_bit(7, true);
        self.control_reg.write(control_reg);
    }
}

/// The Multipurpose Clock Generator
#[repr(C, packed)]
pub struct Mcg {
    c1: Volatile<u8>,
    c2: Volatile<u8>,
    c3: Volatile<u8>,
    c4: Volatile<u8>,
    c5: Volatile<u8>,
    c6: Volatile<u8>,
    s: Volatile<u8>,
    _pad0: u8,
    sc: Volatile<u8>,
    _pad1: u8,
    atcvh: Volatile<u8>,
    atcvl: Volatile<u8>,
    c7: Volatile<u8>,
    c8: Volatile<u8>,
}

impl Mcg {
    pub unsafe fn new() -> &'static mut Mcg {
        &mut *(0x4006_4000 as *mut Mcg)
    }

    pub fn move_to_external_clock(&mut self) {
        /*
         * To move to using the external clock, we go:
         *      FEI -> FBE -> PBE
         *
         * TODO: this assumes the MCG is actually in FEI mode. Check the status registers to make
         * sure this is true
         */

        // We start by enabling the external crystal oscillator
        self.c2.update(|c2| {
            c2.set_bits(4..6, 2);
            c2.set_bit(2, true);
        });

        // Wait for it to become enabled
        while !self.s.read().get_bit(1) {}

        // Move to FBE mode to begin using the external oscillator
        self.c1.update(|c1| {
            c1.set_bits(6..8, 2); // Use external oscillator source
            c1.set_bits(3..6, 4); // Divide it by 512
            c1.set_bit(2, false);
        });

        // Wait for the new clock to stabilise by waiting for the FLL to be pointed at the crystal,
        // then wait for the clock source to change to the crystal oscillator
        while self.s.read().get_bit(4) {}
        while self.s.read().get_bits(2..4) != 2 {}

        // We can now transition to PBE mode by enabling the PLL.
        // We run the PLL at 72Mhz (27/6 * 16 Mhz)
        const NUMERATOR: u8 = 27;
        const DENOMINATOR: u8 = 6;

        self.c5.update(|c5| {
            c5.set_bits(0..5, DENOMINATOR - 1);
        });

        self.c6.update(|c6| {
            c6.set_bits(0..5, NUMERATOR - 24);
            c6.set_bit(6, true);
        });

        // Wait for PLL to be enabled, then for it to become "locked" and stabilise
        while !self.s.read().get_bit(5) {}
        while !self.s.read().get_bit(6) {}

        // Move to using the PLL
        self.c1.update(|c1| {
            c1.set_bits(6..8, 0); // Set the oscillator source to Locked Loop
        });

        while self.s.read().get_bits(2..4) != 3 {}
    }
}
