//! A dummy I2C client

use core::cell::Cell;
use kernel::hil;
use nrf51::i2c;

// ===========================================
// Scan for I2C Slaves
// ===========================================

struct ScanClient {
    dev_id: Cell<u8>,
}

static mut SCAN_CLIENT: ScanClient = ScanClient { dev_id: Cell::new(1) };

impl hil::i2c::I2CClient for ScanClient {
    fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {
        let mut dev_id = self.dev_id.get();

        match error {
            hil::i2c::Error::CommandComplete => println!("0x{:x}", dev_id),
            _ => {}
        }
        let dev = unsafe { &mut i2c::I2C0 };
        if dev_id < 0x7F {
            dev_id += 1;
            self.dev_id.set(dev_id);
            dev.write(dev_id as u32, buffer, 1);
        } else {
            println!("Done scanning for I2C devices. Buffer len: {}",
                     buffer.len());
        }
    }
}

pub fn i2c_scan_slaves() {
    static mut DATA: [u8; 255] = [0; 255];

    let dev = unsafe { &mut i2c::I2C0 };

    let i2c_client = unsafe { &SCAN_CLIENT };
    // dev.set_master_client(i2c_client);

    // let dev: &mut I2CMaster = dev;
    // dev.enable();

    println!("Scanning for I2C devices...");
    dev.write(i2c_client.dev_id.get() as u32, unsafe { &mut DATA }, 2);
}
