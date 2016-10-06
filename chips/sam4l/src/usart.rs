use core::mem;
use core::cell::Cell;

use kernel::common::take_cell::TakeCell;
use dma::{DMAChannel, DMAClient, DMAPeripheral};
use helpers::*;
use kernel::hil::{self, uart, Controller};
use kernel::hil::uart::{Parity, Mode};
use nvic;
use pm::{self, Clock, PBAClock};

#[repr(C, packed)]
struct Registers {
    cr: u32,
    mr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    csr: u32,
    rhr: u32,
    thr: u32,
    brgr: u32, // 0x20
    rtor: u32,
    ttgr: u32,
    reserved0: [u32; 5],
    fidi: u32, // 0x40
    ner: u32,
    reserved1: u32,
    ifr: u32,
    man: u32,
    linmr: u32,
    linir: u32,
    linbrr: u32,
    wpmr: u32,
    wpsr: u32,
    version: u32,
}

const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40024000;

#[derive(Copy,Clone)]
pub enum Location {
    USART0,
    USART1,
    USART2,
    USART3,
}

enum UsartClient<'a> {
    Uart(&'a uart::Client),
    SpiMaster(&'a hil::spi::SpiMasterClient),
}

#[derive(Copy,Clone)]
enum UsartMode {
    Uart,
    Spi,
    Unused,
}

pub struct USART {
    regs: *mut Registers,
    client: TakeCell<UsartClient<'static>>,
    clock: Clock,
    nvic: nvic::NvicIdx,
    dma_peripheral: DMAPeripheral,
    dma: Option<&'static mut DMAChannel>,

    usart_mode: Cell<UsartMode>,

    spi_len: Cell<usize>,
    spi_idx: Cell<usize>,
    spi_write_buffer: TakeCell<&'static mut [u8]>,
    spi_read_buffer: TakeCell<Option<&'static mut [u8]>>,
}

pub struct USARTParams {
    // pub client: &'static Shared<uart::Client>,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub mode: Mode,
}

impl Controller for USART {
    type Config = USARTParams;

    fn configure(&self, params: USARTParams) {
        //   self.client = Some(params.client.borrow_mut());
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode =
            (params.mode as u32) /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12 /* Number of stop bits = 1 */
            | 1 << 19 /* Oversample at 8 times baud rate */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.ttgr, 4);
        self.enable_rx_interrupts();
    }
}

pub static mut USART0: USART =
    USART::new(Location::USART0, PBAClock::USART0, nvic::NvicIdx::USART0);
pub static mut USART1: USART =
    USART::new(Location::USART1, PBAClock::USART1, nvic::NvicIdx::USART1);
pub static mut USART2: USART =
    USART::new(Location::USART2, PBAClock::USART2, nvic::NvicIdx::USART2);
pub static mut USART3: USART =
    USART::new(Location::USART3, PBAClock::USART3, nvic::NvicIdx::USART3);

impl USART {
    const fn new(location: Location, clock: PBAClock, nvic: nvic::NvicIdx) -> USART {
        USART {
            regs: (BASE_ADDRESS + (location as usize) * SIZE) as *mut Registers,
            clock: Clock::PBA(clock),
            nvic: nvic,
            dma: None,
            dma_peripheral: DMAPeripheral::USART0_RX, // Set to some default.
            // This is updated when a
            // real DMA is configured.
            client: TakeCell::empty(),

            usart_mode: Cell::new(UsartMode::Unused),

            spi_len: Cell::new(0),
            spi_idx: Cell::new(0),
            spi_write_buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
        }
    }

    fn set_internal_client(&self, client: UsartClient<'static>) {
        self.client.replace(client);
    }

    pub fn set_uart_client(&self, client: &'static hil::uart::Client) {
        let k = UsartClient::Uart(client);
        self.set_internal_client(k);
        // USART::set_client(self, k);
    }

    pub fn set_dma(&mut self, dma: &'static mut DMAChannel, dma_peripheral: DMAPeripheral) {
        self.dma = Some(dma);
        self.dma_peripheral = dma_peripheral;
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let cd = 48000000 / (8 * baud_rate);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.brgr, cd);
    }

    fn set_mode(&self, mode: u32) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.mr, mode);
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);
        }
    }

    fn enable_nvic(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_nvic(&self) {
        unsafe {
            nvic::disable(self.nvic);
        }
    }

    pub fn enable_rx_interrupts(&self) {
        self.enable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.ier, 1 as u32);
    }

    pub fn enable_tx_interrupts(&mut self) {
        self.enable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.ier, 2 as u32);
    }

    pub fn disable_rx_interrupts(&mut self) {
        self.disable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.idr, 1 as u32);
    }

    pub fn handle_interrupt(&mut self) {


        use kernel::hil::uart::UART;
        if self.rx_ready() {



            let regs: &Registers = unsafe { mem::transmute(self.regs) };
            let c = read_volatile(&regs.rhr) as u8;

            // let mode = read_volatile(&regs.mr);
            // panic!("mode {}", self.usart_mode.get() as u8);


            // match self.client {

                // Some(client) => {

            match self.usart_mode.get() {
                UsartMode::Uart => {
                    self.client.map(|usartclient| {
                        match usartclient {
                            &mut UsartClient::Uart(client) => client.read_done(c),
                            &mut UsartClient::SpiMaster(client) => {},
                        }
                    });
                }
                UsartMode::Spi => {
                    self.spi_got_byte(c);
                }
                _ => {}
            }


                // if mode & 0xF == 0xE {
                //     // SPI

                // } else if (mode & 0xF == 0x0) || (mode & 0xF == 0x2) {

                // }


                // }

                    // client.read_done(c),
                // None => {}
            // });
        }
    }

    pub fn reset_rx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, 1 << 2);
    }

    fn spi_got_byte(&self, byte: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
// panic!("gob");

        let len = self.spi_len.get();
        let idx = self.spi_idx.get();

        self.spi_read_buffer.take().map(|read_buffer_option| {



            if read_buffer_option.is_some() {
                read_buffer_option.map(|rbuf| {

                    rbuf[idx] = byte;

                    if idx == len - 1 {
                        // done
        // panic!("are we ok");

                        // cs high
                        write_volatile(&mut regs.cr, (1 << 19));

                        write_volatile(&mut regs.cr, (1 << 5) | (1 << 7));

                        self.spi_write_buffer.take().map(|write_buffer| {

                            self.client.map(move |usartclient| {

                                match usartclient {
                                    &mut UsartClient::Uart(client) => {}
                                    &mut UsartClient::SpiMaster(client) => {
                                        client.read_write_done(write_buffer, Some(rbuf), len);
                                    }
                                }
                            });
                        });
                    } else {
                        self.spi_idx.set(idx+1);

                        self.spi_write_buffer.map(|write_buffer| {
                            write_volatile(&mut regs.thr, write_buffer[idx+1] as u32);
                        });

                        self.spi_read_buffer.replace(Some(rbuf));


                    }

                    // while read_volatile(&mut regs.csr) & 0x2 == 0 {}


                    // // cs low
                    // write_volatile(&mut regs.cr, (1 << 18));

                    // for i in 0..len {
                    //     write_volatile(&mut regs.thr, write_buffer[i] as u32);

                    //     while read_volatile(&mut regs.csr) & 0x1 == 0 {}

                    //     rbuf[i] = read_volatile(&mut regs.rhr) as u8;
                    // }




                });
            } else {
                // Just go ahead and put this back...
                self.spi_read_buffer.replace(read_buffer_option);

                if idx == len - 1 {
                    // done

                    // cs high
                    write_volatile(&mut regs.cr, (1 << 19));

                    // disable
                    write_volatile(&mut regs.cr, (1 << 5) | (1 << 7));

                    self.spi_write_buffer.take().map(|write_buffer| {

                        self.client.map(move |usartclient| {
                            match usartclient {
                                &mut UsartClient::Uart(client) => {}
                                &mut UsartClient::SpiMaster(client) => {
                                    client.read_write_done(write_buffer, None, len);
                                }
                            }
                        });
                    });
                } else {
                    self.spi_idx.set(idx+1);

                    self.spi_write_buffer.map(|write_buffer| {
                        write_volatile(&mut regs.thr, write_buffer[idx+1] as u32);
                    });


                }

                // while read_volatile(&mut regs.csr) & 0x2 == 0 {}

                // // panic!("tx ready");

                // // cs low
                // write_volatile(&mut regs.cr, (1 << 18));

                // for i in 0..len {
                //     write_volatile(&mut regs.thr, write_buffer[i] as u32);

                //     while read_volatile(&mut regs.csr) & 0x1 == 0 {}

                //     let k = read_volatile(&mut regs.rhr);
                // }

                // // cs high
                // write_volatile(&mut regs.cr, (1 << 19));

                // write_volatile(&mut regs.cr, (1 << 5) | (1 << 7));

                // self.client.map(move |usartclient| {
                //     match usartclient {
                //         &mut UsartClient::Uart(client) => {}
                //         &mut UsartClient::SpiMaster(client) => {
                //             client.read_write_done(write_buffer, None, len);
                //         }
                //     }
                // });
            }
        });
    }
}

impl DMAClient for USART {
    fn xfer_done(&mut self, _pid: usize) {
        let buffer = match self.dma.as_mut() {
            Some(dma) => {
                let buf = dma.abort_xfer();
                dma.disable();
                buf
            }
            None => None,
        };
        self.client.map(|usartclient| {
        // self.client.map(move |client| {
        // self.client.as_ref().map(move |client| {
            buffer.map(|buf| {
                match usartclient {
                    &mut UsartClient::Uart(client) => client.write_done(buf),
                    &mut UsartClient::SpiMaster(client) => {},
                }
                // c.write_done(buf)
            });
        });
    }
}

impl uart::UART for USART {
    fn init(&mut self, params: uart::UARTParams) {
        panic!("now way this gets called {}", self.usart_mode.get() as u8);
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode =
            (params.mode as u32) /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12 /* Number of stop bits = 1 */
            | 1 << 19 /* Oversample at 8 times baud rate */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.ttgr, 4);

        self.usart_mode.set(UsartMode::Uart);


    }

    fn send_byte(&self, byte: u8) {
        while !self.tx_ready() {}
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.thr, byte as u32);
    }

    fn send_bytes(&self, bytes: &'static mut [u8], len: usize) {
        self.dma.as_ref().map(move |dma| {
            dma.enable();
            dma.do_xfer(self.dma_peripheral, bytes, len);
        });
    }

    fn rx_ready(&self) -> bool {
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        read_volatile(&regs.csr) & 0b1 != 0
    }

    fn tx_ready(&self) -> bool {
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        read_volatile(&regs.csr) & 0b10 != 0
    }


    fn read_byte(&self) -> u8 {
        while !self.rx_ready() {}
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        read_volatile(&regs.rhr) as u8
    }

    fn enable_rx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, 1 << 4);
    }

    fn disable_rx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, 1 << 5);
    }

    fn enable_tx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, 1 << 6);
    }

    fn disable_tx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, 1 << 7);
    }
}

impl hil::spi::SpiMaster for USART {
    fn init(&self) {

        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };


        // let chrl = ((params.data_bits - 1) & 0x3) as u32;


        self.usart_mode.set(UsartMode::Spi);


        self.enable_clock();

        // Set baud rate. Different math for SPI
        let cd = 16000000 / (2000000);
        write_volatile(&mut regs.brgr, cd);
        // self.set_baud_rate(params.baud_rate);


        let mode =
            0xe << 0 /* SPI Master mode */
            | 0 << 4 /* USCLKS*/
            | 0x3 << 6 /* Character Length 8 bits */
            | 0x4 << 9 /* No Parity */
            | 1 << 18 /* USART drives the clock pin */;
        self.set_mode(mode);

        // Disable transmitter timeguard
        write_volatile(&mut regs.ttgr, 4);



    }


    fn set_client(&self, client: &'static hil::spi::SpiMasterClient) {
        let k = UsartClient::SpiMaster(client);
        self.set_internal_client(k);
        // USART::set_client(self, k);
    }

    fn is_busy(&self) -> bool {
        return false;
    }

    fn read_write_bytes(&self,
                        mut write_buffer: &'static mut [u8],
                        mut read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> bool {


        self.enable_rx_interrupts();

        // enable rx and tx
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, (1 << 4) | (1 << 6));

    // if len == 5 {
    //     panic!("5555555");
    // }


        self.spi_len.set(len);
        self.spi_idx.set(0);

        // cs low
        write_volatile(&mut regs.cr, (1 << 18));


        write_volatile(&mut regs.thr, write_buffer[0] as u32);
        self.spi_write_buffer.replace(write_buffer);
        self.spi_read_buffer.replace(read_buffer);





        // if read_buffer.is_some() {
        //     read_buffer.map(|rbuf| {

        //         while read_volatile(&mut regs.csr) & 0x2 == 0 {}


        //         // cs low
        //         write_volatile(&mut regs.cr, (1 << 18));

        //         for i in 0..len {
        //             write_volatile(&mut regs.thr, write_buffer[i] as u32);

        //             while read_volatile(&mut regs.csr) & 0x1 == 0 {}

        //             rbuf[i] = read_volatile(&mut regs.rhr) as u8;
        //         }

        //         // cs high
        //         write_volatile(&mut regs.cr, (1 << 19));

        //         write_volatile(&mut regs.cr, (1 << 5) | (1 << 7));

        //         self.client.map(move |usartclient| {
        //             match usartclient {
        //                 &mut UsartClient::Uart(client) => {}
        //                 &mut UsartClient::SpiMaster(client) => {
        //                     client.read_write_done(write_buffer, Some(rbuf), len);
        //                 }
        //             }
        //         });
        //     });
        // } else {

        //     while read_volatile(&mut regs.csr) & 0x2 == 0 {}

        //     // panic!("tx ready");

        //     // cs low
        //     write_volatile(&mut regs.cr, (1 << 18));

        //     for i in 0..len {
        //         write_volatile(&mut regs.thr, write_buffer[i] as u32);

        //         while read_volatile(&mut regs.csr) & 0x1 == 0 {}

        //         let k = read_volatile(&mut regs.rhr);
        //     }

        //     // cs high
        //     write_volatile(&mut regs.cr, (1 << 19));

        //     write_volatile(&mut regs.cr, (1 << 5) | (1 << 7));

        //     self.client.map(move |usartclient| {
        //         match usartclient {
        //             &mut UsartClient::Uart(client) => {}
        //             &mut UsartClient::SpiMaster(client) => {
        //                 client.read_write_done(write_buffer, None, len);
        //             }
        //         }
        //     });
        // }




        // write_volatile(&mut regs.thr, 0x9);

        return false;
    }
    fn write_byte(&self, val: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        write_volatile(&mut regs.cr, (1 << 4) | (1 << 6));


        write_volatile(&mut regs.thr, 0xa);
    }
    fn read_byte(&self) -> u8 {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        let a = read_volatile(&mut regs.csr);

        panic!("csr {:x}", a);




        return 0;
    }
    fn read_write_byte(&self, val: u8) -> u8 {
        return 0;
    }

    /// Returns whether this chip select is valid and was
    /// applied, 0 is always valid.
    fn set_chip_select(&self, cs: u8) -> bool {
        return false;
    }
    fn clear_chip_select(&self) {}
    fn get_chip_select(&self) -> u8 {
        return 0;
    }

    /// Returns the actual rate set
    fn set_rate(&self, rate: u32) -> u32 {
        return 0;
    }
    fn get_rate(&self) -> u32 {
        return 0;
    }
    fn set_clock(&self, polarity: hil::spi::ClockPolarity) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        let mode = read_volatile(&mut regs.mr);

        match polarity {
            hil::spi::ClockPolarity::IdleLow => {
                write_volatile(&mut regs.mr, mode & !(1 << 16));
            }
            hil::spi::ClockPolarity::IdleHigh => {
                write_volatile(&mut regs.mr, mode | (1 << 16));
            }
        }
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        let mode = read_volatile(&mut regs.mr);

        match mode & (1 << 16) {
            0 => hil::spi::ClockPolarity::IdleLow,
            _ => hil::spi::ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: hil::spi::ClockPhase) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        let mode = read_volatile(&mut regs.mr);

        match phase {
            hil::spi::ClockPhase::SampleLeading => {
                write_volatile(&mut regs.mr, mode | (1 << 8));
            }
            hil::spi::ClockPhase::SampleTrailing => {
                write_volatile(&mut regs.mr, mode & !(1 << 8));
            }
        }
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        let mode = read_volatile(&mut regs.mr);

        match mode & (1 << 8) {
            0 => hil::spi::ClockPhase::SampleLeading,
            _ => hil::spi::ClockPhase::SampleTrailing,
        }
    }

    // These two functions determine what happens to the chip
    // select line between transfers. If hold_low() is called,
    // then the chip select line is held low after transfers
    // complete. If release_low() is called, then the chip select
    // line is brought high after a transfer completes. A "transfer"
    // is any of the read/read_write calls. These functions
    // allow an application to manually control when the
    // CS line is high or low, such that it can issue multi-byte
    // requests with single byte operations.
    fn hold_low(&self) {}
    fn release_low(&self) {}
}



interrupt_handler!(usart0_handler, USART0);
interrupt_handler!(usart1_handler, USART1);
interrupt_handler!(usart2_handler, USART2);
interrupt_handler!(usart3_handler, USART3);
