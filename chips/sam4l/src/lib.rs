#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core_intrinsics,concat_idents,const_fn)]
#![no_std]

extern crate cortexm4;
#[macro_use]
extern crate kernel;

#[macro_use]
mod helpers;

pub mod chip;
pub mod ast;
pub mod bpm;
pub mod dma;
pub mod i2c;
pub mod spi;
pub mod nvic;
pub mod pm;
pub mod gpio;
pub mod usart;
pub mod scif;
pub mod adc;
pub mod flashcalw;

unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    asm!(
        "mrs    r0, ipsr                    "
        : "={r0}"(interrupt_number)
        :
        : "r0"
        :
        );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined by platform
    fn reset_handler();

    // Defined in src/arch/cortex-m4/ctx_switch.S
    fn SVC_Handler();
    fn systick_handler();

    fn generic_isr();

    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

#[link_section=".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack, reset_handler,
    /* NMI */           unhandled_interrupt,
    /* Hard Fault */    hard_fault_handler,
    /* MemManage */     unhandled_interrupt,
    /* BusFault */      unhandled_interrupt,
    /* UsageFault*/     unhandled_interrupt,
    unhandled_interrupt, unhandled_interrupt, unhandled_interrupt,
    unhandled_interrupt,
    /* SVC */           SVC_Handler,
    /* DebugMon */      unhandled_interrupt,
    unhandled_interrupt,
    /* PendSV */        unhandled_interrupt,
    /* SysTick */       systick_handler
];

#[link_section=".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 80] = [generic_isr; 80];

#[no_mangle]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static INTERRUPT_TABLE: [Option<unsafe extern fn()>; 80] = [
    // Perhipheral vectors are defined by Atmel in the SAM4L datasheet section
    // 4.7.
    /* HFLASHC */       Option::Some(flashcalw::flash_handler),
    /* PDCA0 */         Option::Some(dma::pdca0_handler),
    /* PDCA1 */         Option::Some(dma::pdca1_handler),
    /* PDCA2 */         Option::Some(dma::pdca2_handler),
    /* PDCA3 */         Option::Some(dma::pdca3_handler),
    /* PDCA4 */         Option::Some(dma::pdca4_handler),
    /* PDCA5 */         Option::Some(dma::pdca5_handler),
    /* PDCA6 */         Option::Some(dma::pdca6_handler),
    /* PDCA7 */         Option::Some(dma::pdca7_handler),
    /* PDCA8 */         Option::Some(dma::pdca8_handler),
    /* PDCA9 */         Option::Some(dma::pdca9_handler),
    /* PDCA10 */        Option::Some(dma::pdca10_handler),
    /* PDCA11 */        Option::Some(dma::pdca11_handler),
    /* PDCA12 */        Option::Some(dma::pdca12_handler),
    /* PDCA13 */        Option::Some(dma::pdca13_handler),
    /* PDCA14 */        Option::Some(dma::pdca14_handler),
    /* PDCA15 */        Option::Some(dma::pdca15_handler),
    /* CRCCU */         Option::Some(unhandled_interrupt),
    /* USBC */          Option::Some(unhandled_interrupt),
    /* PEVC_TR */       Option::Some(unhandled_interrupt),
    /* PEVC_OV */       Option::Some(unhandled_interrupt),
    /* AESA */          Option::Some(unhandled_interrupt),
    /* PM */            Option::Some(unhandled_interrupt),
    /* SCIF */          Option::Some(unhandled_interrupt),
    /* FREQM */         Option::Some(unhandled_interrupt),
    /* GPIO0 */         Option::Some(gpio::gpio0_handler),
    /* GPIO1 */         Option::Some(gpio::gpio1_handler),
    /* GPIO2 */         Option::Some(gpio::gpio2_handler),
    /* GPIO3 */         Option::Some(gpio::gpio3_handler),
    /* GPIO4 */         Option::Some(gpio::gpio4_handler),
    /* GPIO5 */         Option::Some(gpio::gpio5_handler),
    /* GPIO6 */         Option::Some(gpio::gpio6_handler),
    /* GPIO7 */         Option::Some(gpio::gpio7_handler),
    /* GPIO8 */         Option::Some(gpio::gpio8_handler),
    /* GPIO9 */         Option::Some(gpio::gpio9_handler),
    /* GPIO10 */        Option::Some(gpio::gpio10_handler),
    /* GPIO11 */        Option::Some(gpio::gpio11_handler),
    /* BPM */           Option::Some(unhandled_interrupt),
    /* BSCIF */         Option::Some(unhandled_interrupt),
    /* AST_ALARM */     Option::Some(ast::ast_alarm_handler),
    /* AST_PER */       Option::Some(unhandled_interrupt),
    /* AST_OVF */       Option::Some(unhandled_interrupt),
    /* AST_READY */     Option::Some(unhandled_interrupt),
    /* AST_CLKREADY */  Option::Some(unhandled_interrupt),
    /* WDT */           Option::Some(unhandled_interrupt),
    /* EIC1 */          Option::Some(unhandled_interrupt),
    /* EIC2 */          Option::Some(unhandled_interrupt),
    /* EIC3 */          Option::Some(unhandled_interrupt),
    /* EIC4 */          Option::Some(unhandled_interrupt),
    /* EIC5 */          Option::Some(unhandled_interrupt),
    /* EIC6 */          Option::Some(unhandled_interrupt),
    /* EIC7 */          Option::Some(unhandled_interrupt),
    /* EIC8 */          Option::Some(unhandled_interrupt),
    /* IISC */          Option::Some(unhandled_interrupt),
    /* SPI */           Option::Some(unhandled_interrupt),
    /* TC00 */          Option::Some(unhandled_interrupt),
    /* TC01 */          Option::Some(unhandled_interrupt),
    /* TC02 */          Option::Some(unhandled_interrupt),
    /* TC10 */          Option::Some(unhandled_interrupt),
    /* TC11 */          Option::Some(unhandled_interrupt),
    /* TC12 */          Option::Some(unhandled_interrupt),
    /* TWIM0 */         Option::Some(i2c::twim0_handler),
    /* TWIS0 */         Option::Some(unhandled_interrupt),
    /* TWIM1 */         Option::Some(i2c::twim1_handler),
    /* TWIS1 */         Option::Some(unhandled_interrupt),
    /* USART0 */        Option::Some(usart::usart0_handler),
    /* USART1 */        Option::Some(usart::usart1_handler),
    /* USART2 */        Option::Some(usart::usart2_handler),
    /* USART3 */        Option::Some(usart::usart3_handler),
    /* ADCIFE */        Option::Some(adc::adcife_handler),
    /* DACC */          Option::Some(unhandled_interrupt),
    /* ACIFC */         Option::Some(unhandled_interrupt),
    /* ABDACB */        Option::Some(unhandled_interrupt),
    /* TRNG */          Option::Some(unhandled_interrupt),
    /* PARC */          Option::Some(unhandled_interrupt),
    /* CATB */          Option::Some(unhandled_interrupt),
    None,
    /* TWIM2 */         Option::Some(i2c::twim2_handler),
    /* TWIM3 */         Option::Some(i2c::twim3_handler),
    /* LCDCA */         Option::Some(unhandled_interrupt),
];

pub unsafe fn init() {

    // Relocate data segment.
    // Assumes data starts right after text segment as specified by the linker
    // file.
    let mut pdest = &mut _srelocate as *mut u32;
    let pend = &mut _erelocate as *mut u32;
    let mut psrc = &_etext as *const u32;

    if psrc != pdest {
        while (pdest as *const u32) < pend {
            *pdest = *psrc;
            pdest = pdest.offset(1);
            psrc = psrc.offset(1);
        }
    }

    // Clear the zero segment (BSS)
    let pzero = &_ezero as *const u32;
    pdest = &mut _szero as *mut u32;

    while (pdest as *const u32) < pzero {
        *pdest = 0;
        pdest = pdest.offset(1);
    }



    // // 12 mghz from amit

    // // unlock
    // ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000048);
    // // enable 12 Mhz RC oscillator
    // ::core::intrinsics::volatile_store(0x400E0848 as *mut usize, 0b10 << 8 | 1);
    // while ::core::intrinsics::volatile_load(0x400E0848 as *const usize) & 1 == 0 {}
    // pm::select_main_clock(pm::MainClock::RCFAST);

    // return;

// panic!("odd");





    // 48 MHz DLL from Michael
    // Following the `sysclk_init()` code from ASF and these `defines`:
    // #define CONFIG_SYSCLK_SOURCE        SYSCLK_SRC_DFLL
    // #define CONFIG_HCACHE_ENABLE          1
    // #define CONFIG_FLASH_READ_MODE_HIGH_SPEED_ENABLE
    // #define CONFIG_SYSCLK_CPU_DIV         0
    // #define CONFIG_SYSCLK_PBA_DIV         0
    // #define CONFIG_SYSCLK_PBB_DIV         0
    // #define CONFIG_SYSCLK_PBC_DIV         0
    // #define CONFIG_SYSCLK_PBD_DIV         0
    // #define CONFIG_DFLL0_SOURCE         GENCLK_SRC_RC32K
    // #define CONFIG_DFLL0_FREQ           48000000UL
    // #define CONFIG_DFLL0_MUL            (CONFIG_DFLL0_FREQ / OSC_RC32K_NOMINAL_HZ)
    // #define CONFIG_DFLL0_DIV            1

    // TODO: Enable HCACHE
    // sysclk_enable_peripheral_clock(HCACHE);
    // HCACHE->HCACHE_CTRL = HCACHE_CTRL_CEN_YES;
    // while (!(HCACHE->HCACHE_SR & HCACHE_SR_CSTS_EN));
    pm::enable_clock(pm::Clock::HSB(pm::HSBClock::FLASHCALWP));
    pm::enable_clock(pm::Clock::PBB(pm::PBBClock::HRAMC1));
    ::core::intrinsics::volatile_store(0x400A0408 as *mut usize, 1);
    while ::core::intrinsics::volatile_load(0x400A040c as *const usize) & (1 << 0) == 0 {}

    // Using GENCLK_SRC_RC32K as the source for the DFLL. Must enable it first.
    //      BSCIF->BSCIF_UNLOCK = BSCIF_UNLOCK_KEY(0xAAu) | RC32KCR_OFFSET;
    //      BSCIF->BSCIF_RC32KCR = temp | BSCIF_RC32KCR_EN32K | BSCIF_RC32KCR_EN;
    // Get the original value
    let bscif_rc32kcr = ::core::intrinsics::volatile_load(0x400F0424 as *const usize);
    // Unlock the BSCIF::RC32KCR register
    ::core::intrinsics::volatile_store(0x400F0418 as *mut usize, 0xAA000024);
    // Write the BSCIF::RC32KCR register
    ::core::intrinsics::volatile_store(0x400F0424 as *mut usize, bscif_rc32kcr | (1 << 2) | (1 << 0));
    // Wait for it to be ready, although it feels like this won't do anything
    while ::core::intrinsics::volatile_load(0x400F0424 as *const usize) & (1 << 0) == 0 {}



    // Next init closed loop mode.
    // For some reason do a SCIF sync before reading the SCIF register
    ::core::intrinsics::volatile_store(0x400E0840 as *mut usize, 1);
    // Wait for it to be ready
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 3) == 0 {}

    // Read the current DFLL settings
    let scif_dfll0conf = ::core::intrinsics::volatile_load(0x400E0828 as *const usize);
    // Set the new values
    //                                        enable     closed loop
    let scif_dfll0conf_new1 = scif_dfll0conf | (1 << 0) | (1 << 1);
    // let scif_dfll0conf_new1 = (1 << 0) | (1 << 1);
    // let scif_dfll0conf_new1 = (1 << 0) | (1 << 1) | (1 << 20);
    let scif_dfll0conf_new2 = scif_dfll0conf_new1 & (!(3 << 16));
    let scif_dfll0conf_new3 = scif_dfll0conf_new2 | (2 << 16); // frequency range 2
    // Enable the general clock. Yeah getting this fields is complicated.
    //                 enable     RC32K       no divider
    let scif_gcctrl0 = (1 << 0) | (13 << 8) | (0 << 1) | (0 << 16);
    ::core::intrinsics::volatile_store(0x400E0874 as *mut usize, scif_gcctrl0);
    // First, enable dfll apparently
    // unlock dfll0conf
    ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000028);
    // enable
    ::core::intrinsics::volatile_store(0x400E0828 as *mut usize, 1);
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 3) == 0 {}
    // Set step values
    // unlock
    ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000034);
    // 4, 4
    ::core::intrinsics::volatile_store(0x400E0834 as *mut usize, (4 << 0) | (4 << 16));
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 3) == 0 {}
    // Set multiply value
    // unlock
    ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000030);
    // 1464 = 48000000 / 32768
    // ::core::intrinsics::volatile_store(0x400E0830 as *mut usize, 1464);
    // ::core::intrinsics::volatile_store(0x400E0830 as *mut usize, 1500);
    ::core::intrinsics::volatile_store(0x400E0830 as *mut usize, 1563);
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 3) == 0 {}
    // Set SSG value
    // unlock
    ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000038);
    // just set to zero to disable
    ::core::intrinsics::volatile_store(0x400E0838 as *mut usize, 0);
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 3) == 0 {}
    // Set actual configuration
    // unlock
    ::core::intrinsics::volatile_store(0x400E0818 as *mut usize, 0xAA000028);
    // we already prepared this value
    ::core::intrinsics::volatile_store(0x400E0828 as *mut usize, scif_dfll0conf_new3);

    // Now wait for it to be ready (DFLL0LOCKF)
    while ::core::intrinsics::volatile_load(0x400E0814 as *const usize) & (1 << 2) == 0 {}



    // Since we are running at a fast speed we have to set a clock delay
    // for flash, as well as enable fast flash mode.
    let flashcalw_fcr = ::core::intrinsics::volatile_load(0x400A0000 as *const usize);
    ::core::intrinsics::volatile_store(0x400A0000 as *mut usize, flashcalw_fcr | (1 << 6));

    // Enable high speed mode for flash
    let flashcalw_fcmd = ::core::intrinsics::volatile_load(0x400A0004 as *const usize);
    let flashcalw_fcmd_new1 = flashcalw_fcmd & (!(0x3F << 0));
    let flashcalw_fcmd_new2 = flashcalw_fcmd_new1 | (0xA5 << 24) | (0x10 << 0);
    ::core::intrinsics::volatile_store(0x400A0004 as *mut usize, flashcalw_fcmd_new2);

    // And wait for the flash to be ready
    while ::core::intrinsics::volatile_load(0x400A0008 as *const usize) & (1 << 0) == 0 {}

    // TODO: run bpm_configure_power_scaling()

    // Choose the main clock in the PM module.
    pm::select_main_clock(pm::MainClock::DFLL);
// panic!("odd");
}

unsafe extern "C" fn hard_fault_handler() {
    use core::intrinsics::offset;

    let faulting_stack: *mut u32;
    let kernel_stack: bool;

    asm!(
        "mov    r1, 0                       \n\
         tst    lr, #4                      \n\
         itte   eq                          \n\
         mrseq  r0, msp                     \n\
         addeq  r1, 1                       \n\
         mrsne  r0, psp                     "
        : "={r0}"(faulting_stack), "={r1}"(kernel_stack)
        :
        : "r0", "r1"
        :
        );

    let stacked_r0: u32 = *offset(faulting_stack, 0);
    let stacked_r1: u32 = *offset(faulting_stack, 1);
    let stacked_r2: u32 = *offset(faulting_stack, 2);
    let stacked_r3: u32 = *offset(faulting_stack, 3);
    let stacked_r12: u32 = *offset(faulting_stack, 4);
    let stacked_lr: u32 = *offset(faulting_stack, 5);
    let stacked_pc: u32 = *offset(faulting_stack, 6);
    let stacked_prs: u32 = *offset(faulting_stack, 7);

    let mode_str = if kernel_stack { "Kernel" } else { "Process" };

    let shcsr: u32 = core::ptr::read_volatile(0xE000ED24 as *const u32);
    let cfsr: u32 = core::ptr::read_volatile(0xE000ED28 as *const u32);
    let hfsr: u32 = core::ptr::read_volatile(0xE000ED2C as *const u32);

    panic!("{} HardFault.\n\
           \tr0  0x{:x}\n\
           \tr1  0x{:x}\n\
           \tr2  0x{:x}\n\
           \tr3  0x{:x}\n\
           \tr12 0x{:x}\n\
           \tlr  0x{:x}\n\
           \tpc  0x{:x}\n\
           \tprs 0x{:x}\n\
           \tsp  0x{:x}\n\
           \tSHCSR 0x{:x}\n\
           \tCFSR  0x{:x}\n\
           \tHSFR  0x{:x}\n\
           ", mode_str,
           stacked_r0, stacked_r1, stacked_r2, stacked_r3,
           stacked_r12, stacked_lr, stacked_pc, stacked_prs,
           faulting_stack as u32, shcsr, cfsr, hfsr);
}
