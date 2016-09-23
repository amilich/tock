use core::cell::Cell;
use core::intrinsics;
use core::mem;

use helpers::*;
use kernel::common::take_cell::TakeCell;
use nvic;
use pm;

/// Memory registers for a DMA channel. Section 16.6.1 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct DMARegisters {
    memory_address: usize,
    peripheral_select: usize,
    transfer_counter: usize,
    memory_address_reload: usize,
    transfer_counter_reload: usize,
    control: usize,
    mode: usize,
    status: usize,
    interrupt_enable: usize,
    interrupt_disable: usize,
    interrupt_mask: usize,
    interrupt_status: usize,
    version: usize,
    _unused: [usize; 3],
}

/// The PDCA's base addresses in memory (Section 7.1 of manual)
pub const DMA_BASE_ADDR: usize = 0x400A2000;

/// The number of bytes between each memory mapped DMA Channel (Section 16.6.1)
pub const DMA_CHANNEL_SIZE: usize = 0x40;

/// Shared counter that Keeps track of how many DMA channels are currently
/// active.
static mut NUM_ENABLED: usize = 0;

/// The DMA channel number. Each channel transfers data between memory and a
/// particular peripheral function (e.g., SPI read or SPI write, but not both
/// simultaneously). There are 16 available channels (Section 16.7)
#[derive(Copy,Clone)]
pub enum DMAChannelNum {
    // Relies on the fact that assigns values 0-15 to each constructor in order
    DMAChannel00 = 0,
    DMAChannel01 = 1,
    DMAChannel02 = 2,
    DMAChannel03 = 3,
    DMAChannel04 = 4,
    DMAChannel05 = 5,
    DMAChannel06 = 6,
    DMAChannel07 = 7,
    DMAChannel08 = 8,
    DMAChannel09 = 9,
    DMAChannel10 = 10,
    DMAChannel11 = 11,
    DMAChannel12 = 12,
    DMAChannel13 = 13,
    DMAChannel14 = 14,
    DMAChannel15 = 15,
}


/// The peripheral function a channel is assigned to (Section 16.7)
/// *_RX means transfer data from peripheral to memory, *_TX means transfer data
/// from memory to peripheral.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum DMAPeripheral {
    USART0_RX = 0,
    USART1_RX = 1,
    USART2_RX = 2,
    USART3_RX = 3,
    SPI_RX = 4,
    TWIM0_RX = 5,
    TWIM1_RX = 6,
    TWIM2_RX = 7,
    TWIM3_RX = 8,
    TWIS0_RX = 9,
    TWIS1_RX = 10,
    ADCIFE_RX = 11,
    CATB_RX = 12,
    IISC_CH0_RX = 14,
    IISC_CH1_RX = 15,
    PARC_RX = 16,
    AESA_RX = 17,
    USART0_TX = 18,
    USART1_TX = 19,
    USART2_TX = 20,
    USART3_TX = 21,
    SPI_TX = 22,
    TWIM0_TX = 23,
    TWIM1_TX = 24,
    TWIM2_TX = 25,
    TWIM3_TX = 26,
    TWIS0_TX = 27,
    TWIS1_TX = 28,
    ADCIFE_TX = 29,
    CATB_TX = 30,
    ABDACB_SDR0_TX = 31,
    ABDACB_SDR1_TX = 32,
    IISC_CH0_TX = 33,
    IISC_CH1_TX = 34,
    DACC_TX = 35,
    AESA_TX = 36,
    LCDCA_ACMDR_TX = 37,
    LCDCA_ABMDR_TX = 38,
}

pub static mut DMAChannels: [DMAChannel; 16] =
    [DMAChannel::new(DMAChannelNum::DMAChannel00, nvic::NvicIdx::PDCA0),
     DMAChannel::new(DMAChannelNum::DMAChannel01, nvic::NvicIdx::PDCA1),
     DMAChannel::new(DMAChannelNum::DMAChannel02, nvic::NvicIdx::PDCA2),
     DMAChannel::new(DMAChannelNum::DMAChannel03, nvic::NvicIdx::PDCA3),
     DMAChannel::new(DMAChannelNum::DMAChannel04, nvic::NvicIdx::PDCA4),
     DMAChannel::new(DMAChannelNum::DMAChannel05, nvic::NvicIdx::PDCA5),
     DMAChannel::new(DMAChannelNum::DMAChannel06, nvic::NvicIdx::PDCA6),
     DMAChannel::new(DMAChannelNum::DMAChannel07, nvic::NvicIdx::PDCA7),
     DMAChannel::new(DMAChannelNum::DMAChannel08, nvic::NvicIdx::PDCA8),
     DMAChannel::new(DMAChannelNum::DMAChannel09, nvic::NvicIdx::PDCA9),
     DMAChannel::new(DMAChannelNum::DMAChannel10, nvic::NvicIdx::PDCA10),
     DMAChannel::new(DMAChannelNum::DMAChannel11, nvic::NvicIdx::PDCA11),
     DMAChannel::new(DMAChannelNum::DMAChannel12, nvic::NvicIdx::PDCA12),
     DMAChannel::new(DMAChannelNum::DMAChannel13, nvic::NvicIdx::PDCA13),
     DMAChannel::new(DMAChannelNum::DMAChannel14, nvic::NvicIdx::PDCA14),
     DMAChannel::new(DMAChannelNum::DMAChannel15, nvic::NvicIdx::PDCA15)];

pub struct DMAChannel {
    registers: *mut DMARegisters,
    nvic: nvic::NvicIdx,
    pub client: Option<&'static mut DMAClient>,
    enabled: Cell<bool>,
    buffer: TakeCell<&'static mut [u8]>,
}

pub trait DMAClient {
    fn xfer_done(&mut self, pid: DMAPeripheral);
}

impl DMAChannel {
    const fn new(channel: DMAChannelNum, nvic: nvic::NvicIdx) -> DMAChannel {
        DMAChannel {
            registers: (DMA_BASE_ADDR + (channel as usize) * DMA_CHANNEL_SIZE) as *mut DMARegisters,
            nvic: nvic,
            client: None,
            enabled: Cell::new(false),
            buffer: TakeCell::empty(),
        }
    }

    pub fn enable(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
            pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
        }
        if !self.enabled.get() {
            unsafe {
                let num_enabled = intrinsics::atomic_xadd(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
            write_volatile(&mut registers.interrupt_disable, 0xffffffff);

            unsafe { nvic::enable(self.nvic) };

            self.enabled.set(true);
        }
    }

    pub fn disable(&self) {
        if self.enabled.get() {
            unsafe {
                let num_enabled = intrinsics::atomic_xsub(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::disable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::disable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
            write_volatile(&mut registers.control, 0x2);
            self.enabled.set(false);
            unsafe {
                nvic::disable(self.nvic);
            }
        }
    }

    pub fn handle_interrupt(&mut self) {
        let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
        let channel_num: usize = read_volatile(&registers.peripheral_select);
        let channel: DMAPeripheral = match(channel_num) {
            (USART0_RX as usize)        => USART0_RX,
            (USART1_RX as usize)        => USART1_RX,
            (USART2_RX as usize)        => USART2_RX,
            (USART3_RX as usize)        => USART3_RX,
            (SPI_RX as usize)           => SPI_RX,
            (TWIM0_RX as usize)         => TWIM0_RX,
            (TWIM1_RX as usize)         => TWIM1_RX,
            (TWIM2_RX as usize)         => TWIM2_RX,
            (TWIM3_RX as usize)         => TWIM3_RX,
            (TWIS0_RX as usize)         => TWIS0_RX,
            (TWIS1_RX as usize)         => TWIS1_RX,
            (ADCIFE_RX as usize)        => ADCIFE_RX,
            (CATB_RX as usize)          => CATB_RX,
            (IISC_CH0_RX as usize)      => IISC_CH0_RX,
            (IISC_CH1_RX as usize)      => IISC_CH1_RX,
            (PARC_RX as usize)          => PARC_RX,
            (AESA_RX as usize)          => AESA_RX,
            (USART0_TX as usize)        => USART0_TX,
            (USART1_TX as usize)        => USART1_TX,
            (USART2_TX as usize)        => USART2_TX,
            (USART3_TX as usize)        => USART3_TX,
            (SPI_TX as usize)           => SPI_TX,
            (TWIM0_TX as usize)         => TWIM0_TX,
            (TWIM1_TX as usize)         => TWIM1_TX,
            (TWIM2_TX as usize)         => TWIM2_TX,
            (TWIM3_TX as usize)         => TWIM3_TX,
            (TWIS0_TX as usize)         => TWIS0_TX,
            (TWIS1_TX as usize)         => TWIS1_TX,
            (ADCIFE_TX as usize)        => ADCIFE_TX,
            (CATB_TX as usize)          => CATB_TX,
            (ABDACB_SDR0_TX as usize)   => ABDACB_SDR0_TX,
            (ABDACB_SDR1_TX as usize)   => ABDACB_SDR1_TX,
            (IISC_CH0_TX as usize)      => IISC_CH0_TX,
            (IISC_CH1_TX as usize)      => IISC_CH1_TX,
            (DACC_TX as usize)          => DACC_TX,
            (AESA_TX as usize)          => AESA_TX,
            (LCDCA_ACMDR_TX as usize)   => LCDCA_ACMDR_TX,
            (LCDCA_ABMDR_TX as usize)   => LCDCA_ABMDR_TX,
        };

        self.client.as_mut().map(|client| {
            client.xfer_done(channel);
        });
    }

    pub fn start_xfer(&self) {
        let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
        write_volatile(&mut registers.control, 0x1);
    }

    pub fn prepare_xfer(&self, pid: DMAPeripheral, buf: &'static mut [u8], mut len: usize) {
        // TODO(alevy): take care of zero length case
        if len > buf.len() {
            len = buf.len();
        }

        let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
        write_volatile(&mut registers.peripheral_select, pid as usize);
        write_volatile(&mut registers.memory_address_reload,
                       &buf[0] as *const u8 as usize);
        write_volatile(&mut registers.transfer_counter_reload, len);

        write_volatile(&mut registers.interrupt_enable, 1 << 1);

        // Store the buffer reference in the TakeCell so it can be returned to
        // the caller in `handle_interrupt`
        self.buffer.replace(buf);
    }

    pub fn do_xfer(&self, pid: DMAPeripheral, buf: &'static mut [u8], len: usize) {
        self.prepare_xfer(pid, buf, len);
        self.start_xfer();
    }

    /// Aborts any current transactions and returns the buffer used in the
    /// transaction.
    pub fn abort_xfer(&self) -> Option<&'static mut [u8]> {
        let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
        write_volatile(&mut registers.interrupt_disable, !0);

        // Reset counter
        write_volatile(&mut registers.transfer_counter, 0);

        self.buffer.take()
    }

    pub fn transfer_counter(&self) -> usize {
        let registers: &mut DMARegisters = unsafe { mem::transmute(self.registers) };
        read_volatile(&registers.transfer_counter)
    }
}

macro_rules! pdca_handler {
    ($name: ident, $nvic: ident, $num: expr) => {
        interrupt_handler!(
            $name,
            $nvic,
            {
                let registers : &mut DMARegisters =
                    mem::transmute(DMAChannels[$num].registers);
                write_volatile(&mut registers.interrupt_disable, 0xffffffff);
            });
    }
}

pdca_handler!(pdca0_handler, PDCA0, 0);
pdca_handler!(pdca1_handler, PDCA1, 1);
pdca_handler!(pdca2_handler, PDCA2, 2);
pdca_handler!(pdca3_handler, PDCA3, 3);
pdca_handler!(pdca4_handler, PDCA4, 4);
pdca_handler!(pdca5_handler, PDCA5, 5);
pdca_handler!(pdca6_handler, PDCA6, 6);
pdca_handler!(pdca7_handler, PDCA7, 7);
pdca_handler!(pdca8_handler, PDCA8, 8);
pdca_handler!(pdca9_handler, PDCA9, 9);
pdca_handler!(pdca10_handler, PDCA10, 10);
pdca_handler!(pdca11_handler, PDCA11, 11);
pdca_handler!(pdca12_handler, PDCA12, 12);
pdca_handler!(pdca13_handler, PDCA13, 13);
pdca_handler!(pdca14_handler, PDCA14, 14);
pdca_handler!(pdca15_handler, PDCA15, 15);
