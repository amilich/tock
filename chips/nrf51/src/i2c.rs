/*

The different signals SCL and SDA associated with the TWI master are mapped to physical pins according to
the configuration specified in the PSELSCL and PSELSDA registers respectively. If a value of 0xFFFFFFFF
is specified in any of these registers, the associated TWI master signal is not connected to any physical pin.
The PSELSCL and PSELSDA registers and their configurations are only used as long as the TWI master
is enabled, and retained only as long as the device is in ON mode. PSELSCL and PSESDA must only be
configured when the TWI is disabled.


Therefore, you must disable all peripherals that have the same ID as the TWI before the TWI can be
configured and used

Table 259: Instances
Base address Peripheral Instance Description
0x40003000 TWI TWI0 I2C compatible Two-Wire Interface
0x40004000 TWI TWI1 I2C compatible Two-Wire Interface
Table 260: Register Overview
Register Offset Description
Tasks
STARTRX 0x000 Start TWI receive sequence
STARTTX 0x008 Start TWI transmit sequence
STOP 0x014 Stop TWI transaction
SUSPEND 0x01C Suspend TWI transaction
RESUME 0x020 Resume TWI transaction
Events
STOPPED 0x104 TWI stopped
RXDREADY 0x108 TWI RXD byte received
TXDSENT 0x11C TWI TXD byte sent
ERROR 0x124 TWI error
BB 0x138 TWI byte boundary, generated before each byte that is sent or received
Registers
SHORTS 0x200 Shortcut register
INTEN 0x300 Enable or disable interrupt
INTENSET 0x304 Enable interrupt
INTENCLR 0x308 Disable interrupt
ERRORSRC 0x4C4 Error source
ENABLE 0x500 Enable TWI
PSELSCL 0x508 Pin select for SCL
PSELSDA 0x50C Pin select for SDA
RXD 0x518 RXD register
TXD 0x51C TXD register
FREQUENCY 0x524 TWI frequency
ADDRESS 0x588 Address used in the TWI transfer

 */


