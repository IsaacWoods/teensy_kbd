use bit_field::BitField;
use volatile::Volatile;

pub enum PortName {
    B,
    C,
}

#[repr(C, packed)]
pub struct Port {
    pcr: [Volatile<u32>; 32],
    gpclr: Volatile<u32>,
    gpchr: Volatile<u32>,
    reserved_0: [u8; 24],
    isfr: Volatile<u32>,
}

pub struct Pin {
    port: *mut Port,
    pin: usize,
}

#[repr(C, packed)]
struct GpioBitband {
    pdor: [Volatile<u32>; 32],
    psor: [Volatile<u32>; 32],
    pcor: [Volatile<u32>; 32],
    ptor: [Volatile<u32>; 32],
    pdir: [Volatile<u32>; 32],
    pddr: [Volatile<u32>; 32],
}

pub struct GpioPin {
    bitband: *mut GpioBitband,
    pin: usize,
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut *match name {
            PortName::B => 0x4004_a000 as *mut Port,
            PortName::C => 0x4004_b000 as *mut Port,
        }
    }

    pub unsafe fn pin(&mut self, p: usize) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn set_pin_mode(&mut self, p: usize, mode: u32) {
        self.pcr[p].update(|pcr| {
            pcr.set_bits(8..11, mode);
        });
    }

    pub fn name(&self) -> PortName {
        let addr = (self as *const Port) as u32;
        match addr {
            0x4004_a000 => PortName::B,
            0x4004_b000 => PortName::C,
            _ => unreachable!(),
        }
    }
}

impl Pin {
    pub fn make_gpio(self) -> GpioPin {
        unsafe {
            let port = &mut *self.port;
            port.set_pin_mode(self.pin, 1);
            GpioPin::new(port.name(), self.pin)
        }
    }

    /// Unsafe because only certain pins can be set as serial. Refer to the device pinout.
    pub unsafe fn set_serial(&mut self) {
        let port = &mut *self.port;
        port.set_pin_mode(self.pin, 3);
    }
}

impl GpioPin {
    pub unsafe fn new(port: PortName, pin: usize) -> GpioPin {
        let bitband = match port {
            PortName::B => 0x43fe_0800 as *mut GpioBitband,
            PortName::C => 0x43fe_1000 as *mut GpioBitband,
        };

        GpioPin { bitband, pin }
    }

    pub fn output(&mut self) {
        unsafe {
            (*self.bitband).pddr[self.pin].write(1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            (*self.bitband).psor[self.pin].write(1);
        }
    }
}
