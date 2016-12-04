// The different signals SCL and SDA associated with the TWI master are mapped to physical pins according to
// the configuration specified in the PSELSCL and PSELSDA registers respectively. If a value of 0xFFFFFFFF
// is specified in any of these registers, the associated TWI master signal is not connected to any physical pin.
// The PSELSCL and PSELSDA registers and their configurations are only used as long as the TWI master
// is enabled, and retained only as long as the device is in ON mode. PSELSCL and PSESDA must only be
// configured when the TWI is disabled.
//
//
// Therefore, you must disable all peripherals that have the same ID as the TWI before the TWI can be
// configured and used
//
// Table 259: Instances
// Base address Peripheral Instance Description
// 0x40003000 TWI TWI0 I2C compatible Two-Wire Interface
// 0x40004000 TWI TWI1 I2C compatible Two-Wire Interface
// Table 260: Register Overview
// Register Offset Description
// Tasks
// STARTRX 0x000 Start TWI receive sequence
// STARTTX 0x008 Start TWI transmit sequence
// STOP 0x014 Stop TWI transaction
// SUSPEND 0x01C Suspend TWI transaction
// RESUME 0x020 Resume TWI transaction
// Events
// STOPPED 0x104 TWI stopped
// RXDREADY 0x108 TWI RXD byte received
// TXDSENT 0x11C TWI TXD byte sent
// ERROR 0x124 TWI error
// BB 0x138 TWI byte boundary, generated before each byte that is sent or received
// Registers
// SHORTS 0x200 Shortcut register
// INTEN 0x300 Enable or disable interrupt
// INTENSET 0x304 Enable interrupt
// INTENCLR 0x308 Disable interrupt
// ERRORSRC 0x4C4 Error source
// ENABLE 0x500 Enable TWI
// PSELSCL 0x508 Pin select for SCL
// PSELSDA 0x50C Pin select for SDA
// RXD 0x518 RXD register
// TXD 0x51C TXD register
// FREQUENCY 0x524 TWI frequency
// ADDRESS 0x588 Address used in the TWI transfer
//
//

// Listing of all registers related to the TWIM peripheral.
// Section ??? of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {

}

// Listing of all registers related to the TWIS peripheral.
// Section ??? of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct TWISRegisters {

}

// Three main I2C speeds
#[derive(Clone,Copy)]
pub enum Speed {
    Standard100k,
    Fast400k,
}

// This represents an abstraction of the peripheral hardware.
pub struct I2CHw {

}

impl I2CHw {
    const fn new() -> I2CHw {

    }
}

impl DMAClient for I2CHw {

}

impl hil::i2c::I2CMaster for I2CHw {
	/// This enables the entire I2C peripheral
    fn enable(&self) {
    }

    /// This disables the entire I2C peripheral
    fn disable(&self) {
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
    }

    fn read(&self, addr: u8, data: &'static mut [u8], len: u8) {
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
    }
}

impl hil::i2c::I2CSlave for I2CHw {
    fn enable(&self) {
    }

    /// This disables the entire I2C peripheral
    fn disable(&self) {
    }

    fn set_address(&self, addr: u8) {
    }

    fn write_receive(&self, data: &'static mut [u8], max_len: u8) {
    }

    fn read_send(&self, data: &'static mut [u8], max_len: u8) {
    }

    fn listen(&self) {
    }
}

impl hil::i2c::I2CMasterSlave for I2CHw {

}