use bit_field::BitField;
use volatile::Volatile;

#[repr(C, packed)]
pub struct Watchdog {
    stctrlh: Volatile<u16>,
    stctrll: Volatile<u16>,
    tovalh: Volatile<u16>,
    tovall: Volatile<u16>,
    winh: Volatile<u16>,
    winl: Volatile<u16>,
    refresh: Volatile<u16>,
    unlock: Volatile<u16>,
    tmrouth: Volatile<u16>,
    tmroutl: Volatile<u16>,
    rstcnt: Volatile<u16>,
    presc: Volatile<u16>,
}

impl Watchdog {
    pub unsafe fn new() -> &'static mut Watchdog {
        &mut *(0x4005_2000 as *mut Watchdog)
    }

    pub fn disable(&mut self) {
        unsafe {
            // Unlock the watchdog so we can change its registers
            self.unlock.write(0xc520);
            self.unlock.write(0xd928);

            // Wait for a bus cycle
            asm!("nop; nop" : : : "memory");

            // Disable the watchdog timer
            self.stctrlh.update(|ctrl| {
                ctrl.set_bit(0, false);
            });
        }
    }
}
