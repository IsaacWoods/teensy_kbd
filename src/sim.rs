use bit_field::BitField;
use volatile::Volatile;

pub enum ClockGate {
    PortC,
}

#[repr(C, packed)]
pub struct Sim {
    sopt1: Volatile<u32>,
    sopt1_cfg: Volatile<u32>,
    _pad0: [u8; 4092],
    sopt2: Volatile<u32>,
    _pad1: [u8; 4],
    sopt4: Volatile<u32>,
    sopt5: Volatile<u32>,
    _pad2: [u8; 4],
    sopt7: Volatile<u32>,
    _pad3: [u8; 8],
    sdid: Volatile<u32>,
    _pad4: [u8; 12],
    scgc4: Volatile<u32>,
    scgc5: Volatile<u32>,
    scgc6: Volatile<u32>,
    scgc7: Volatile<u32>,
    clkdiv1: Volatile<u32>,
    clkviv2: Volatile<u32>,
    fcfg1: Volatile<u32>,
    fcfg2: Volatile<u32>,
    uidh: Volatile<u32>,
    uidmh: Volatile<u32>,
    uidml: Volatile<u32>,
    uidl: Volatile<u32>,
}

impl Sim {
    pub unsafe fn new() -> &'static mut Sim {
        &mut *(0x4004_7000 as *mut Sim)
    }

    pub fn enable_clock_gate(&mut self, gate: ClockGate) {
        unsafe {
            match gate {
                ClockGate::PortC => {
                    self.scgc5.update(|scgc| {
                        scgc.set_bit(11, true);
                    });
                }
            }
        }
    }
}
