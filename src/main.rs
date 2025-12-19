#![no_main]
#![no_std]

// Some panic handler needs to be included. This one halts the processor on panic.
use panic_halt as _;

mod gpio;
mod delay;

use cortex_m_rt::entry;
use snr8503_pac as pac;

/// Initializes the system clock to 96MHz
fn clock_init(peripherals: &pac::Peripherals) {
    // delay_init(96);            /* 延时函数初始化主频96MHz*/
    // In Rust we typically rely on cortex-m::delay or a timer. 
    // Since we don't have the implementation of delay_init, we skip it or assume standard usage.
    // The delay_init call logic has been moved to main() using delay::Delay::new
    
    // SYS_WR_PROTECT = 0x7a83;   /* 解除系统寄存器写保护 */
    peripherals.misc.sys_wr_protect().write(|w| unsafe { w.bits(0x7a83) });

    // SYS_AFE_REG0 |= BIT15;     /* BIT15:PLLPDN */
    // BIT15 = 1 << 15
    peripherals.misc.sys_afe_reg0().modify(|r, w| unsafe { w.bits(r.bits() | (1 << 15)) });

    // SoftDelay(100);            /* 延时100us,等待PLL稳定*/
    // Assuming a conservative cycle count for ~100us before PLL is locked.
    // If running at internal oscillator (e.g. 8-16MHz), 10000 cycles is > 100us.
    // We don't have the delay instance here yet, so we use asm delay.
    cortex_m::asm::delay(10000);

    // SYS_CLK_CFG |= 0x000001ff; /* BIT8:0: CLK_HS,1:PLL  | BIT[7:0]CLK_DIV  | 1ff对应96M时钟 */
    // 0x1ff = 0b1_1111_1111
    peripherals.misc.sys_clk_cfg().modify(|r, w| unsafe { w.bits(r.bits() | 0x000001ff) });

    // SYS_SoftResetModule(0xfff);/* 复位所有外设寄存器 */
    peripherals.misc.sys_sft_rst().write(|w| unsafe { w.bits(0xfff) });
}

/// System initialization function
fn system_init(peripherals: &pac::Peripherals) {
    clock_init(peripherals); /* 时钟初始化 */
}

fn multiplex_nrst(peripherals: &pac::Peripherals) {
    // nRST 复用为 P0.0

    // SYS_PROTECT = 0x7a83;
    peripherals.misc.sys_wr_protect().write(|w| unsafe { w.bits(0x7a83) });
    peripherals.misc.sys_io_cfg().write(|w|unsafe {w.bits(0x20)});
    peripherals.misc.sys_wr_protect().write(|w| unsafe { w.bits(0x0) });
}

// Use `main` as the entry point of this application, which may not return.
#[entry]
fn main() -> ! {
    // initialization
    let peripherals = pac::Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();

    system_init(&peripherals);
    multiplex_nrst(&peripherals);

    // Enable GPIO clock (Bit 7)
    peripherals.misc.sys_clk_fen().modify(|r, w| unsafe { w.bits(r.bits() | (1 << 7)) });

    // Initialize Delay (96MHz system clock)
    let mut delay = delay::Delay::new(core_peripherals.SYST, 48_000_000);

    // Configure P0.0 as Output
    let mut gpio_config = gpio::InitStruct::default();
    gpio_config.pin = 1; // P0.0
    gpio_config.mode = gpio::Mode::Output;
    gpio::init(&peripherals.gpio0, &gpio_config);

    loop {
        // application logic
        gpio::set_bits(&peripherals.gpio0, 1);
        delay.delay_ms(1000); // 1s

        gpio::reset_bits(&peripherals.gpio0, 1);
        delay.delay_ms(1000); // 1s
    }
}
