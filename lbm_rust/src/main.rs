#![no_std]
#![no_main]
use panic_reset as _;

#[cortex_m_rt::entry]
fn entry() -> ! {
    loop {}
}
