//! IS25LP064A: 64Mbit/8Mbyte flash memory
//!
//! https://www.issi.com/WW/pdf/25LP032-64A-B.pdf
//!

use stm32h7xx_hal::{
    gpio::{gpiof, gpiog, Analog, Speed},
    nb::Error as nbError,
    prelude::*,
    rcc,
    xspi::{Config, QspiError, QspiMode, QspiWord},
};

pub type FlashResult<T> = Result<T, QspiError>;
pub type NBFlashResult<T> = stm32h7xx_hal::nb::Result<T, QspiError>;

/// Flash erasure enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashErase {
    ///The whole chip
    Chip,
    ///4Kbyte sector address
    Sector4K(u32),
    ///32Kbyte block address
    Block32K(u32),
    ///64Kbyte block address
    Block64K(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlashState {
    Idle,
    Programming { address: u32, chunk: u32 },
    Erasing(FlashErase),
}

/// Flash memory peripheral
pub struct Flash {
    qspi: stm32h7xx_hal::xspi::Qspi<stm32h7xx_hal::stm32::QUADSPI>,
    state: FlashState,
}

/*
 *
 * 6.1 STATUS REGISTER
 * Status Register Format and Status Register Bit Definitionsare described in Table 6.1 & Table 6.2.
 * 0: WIP write in progress(0 == read, 1 == busy)
 * 1: WEL write enable (1 == enabled)
 * 2-5: BP block protection (0 indicates not protected [default])
 * 6: Quad enable, quad output function enable, (1 = enable)
 * 7: Status register write disable (1 == write protected [0 = default])
 *
 * 6.3 READ REGISTER
 *
 *
 * 8.11 SECTOR ERASE OPERATION (SER, D7h/20h)
 *  * instruction, 3 byte address
 *  * WEL is reset after
 * 8.12 BLOCK ERASE OPERATION (BER32K:52h, BER64K:D8h)
 *  * instruction, 3 byte address
 *  * WEL is reset after
 * 8.13 CHIP ERASE OPERATION (CER, C7h/60h)
 *  * instruction only
 *  * WEL is reset after
 * 8.14 WRITE ENABLE OPERATION (WREN, 06h)
 *  * instruction only
 *  * sets WEL
 * 8.16 READ STATUS REGISTER OPERATION (RDSR, 05h)
 *  * instruction, 1 byte read
*/

impl Flash {
    fn wait(&mut self) {
        while self.qspi.is_busy().is_err() {}
    }

    fn write_complete(&mut self) -> FlashResult<bool> {
        match self.status() {
            Ok(status) => Ok(status & 0x01 == 0),
            Err(e) => return Err(e),
        }
    }

    fn wait_write(&mut self) -> FlashResult<()> {
        loop {
            if self.write_complete()? {
                return Ok(());
            }
        }
    }

    fn write_command(&mut self, cmd: u8) -> FlashResult<()> {
        self.wait();
        self.qspi
            .write_extended(QspiWord::U8(cmd), QspiWord::None, QspiWord::None, &[])
    }

    fn write_reg(&mut self, cmd: u8, data: u8) -> FlashResult<()> {
        self.wait();
        self.qspi
            .write_extended(QspiWord::U8(cmd), QspiWord::None, QspiWord::None, &[data])
    }

    fn enable_write(&mut self) -> FlashResult<()> {
        self.write_command(0x06)
    }

    fn assert_info(&mut self) {
        let mut info: [u8; 3] = [0; 3];
        self.wait();
        self.qspi
            .read_extended(
                QspiWord::U8(0x9F),
                QspiWord::None,
                QspiWord::None,
                0,
                &mut info,
            )
            .unwrap();
        assert_eq!(&info, &[157, 96, 23]);
    }

    fn status(&mut self) -> FlashResult<u8> {
        let mut status: [u8; 1] = [0xFF];
        self.wait();
        self.qspi
            .read_extended(
                QspiWord::U8(0x05),
                QspiWord::None,
                QspiWord::None,
                0,
                &mut status,
            )
            .map(|_| status[0])
    }

    /// Reset the internal state, only if you know what you're doing
    pub unsafe fn reset(&mut self) {
        self.state = FlashState::Idle;
    }

    /// Initialize the flash quad spi interface
    pub fn new(
        regs: stm32h7xx_hal::device::QUADSPI,
        prec: rcc::rec::Qspi,
        clocks: &rcc::CoreClocks,
        pf6: gpiof::PF6<Analog>,
        pf7: gpiof::PF7<Analog>,
        pf8: gpiof::PF8<Analog>,
        pf9: gpiof::PF9<Analog>,
        pf10: gpiof::PF10<Analog>,
        pg6: gpiog::PG6<Analog>,
    ) -> Self {
        let _ncs = pg6.into_alternate_af10().set_speed(Speed::VeryHigh); //QUADSPI_BK1_NCS

        let sck = pf10.into_alternate_af9().set_speed(Speed::VeryHigh);
        let io0 = pf8.into_alternate_af10().set_speed(Speed::VeryHigh);
        let io1 = pf9.into_alternate_af10().set_speed(Speed::VeryHigh);
        let io2 = pf7.into_alternate_af9().set_speed(Speed::VeryHigh);
        let io3 = pf6.into_alternate_af9().set_speed(Speed::VeryHigh);

        let config = Config::new(133.mhz()).mode(QspiMode::OneBit);
        let qspi = regs.bank1((sck, io0, io1, io2, io3), config, &clocks, prec);

        let mut flash = Flash {
            qspi,
            state: FlashState::Idle,
        };

        //enable quad
        flash.enable_write().unwrap();
        flash.write_command(0x35).unwrap();
        flash.qspi.configure_mode(QspiMode::FourBit).unwrap();

        flash.enable_write().unwrap();
        //only enable write, nothing else
        flash.write_reg(0x01, 0b0000_0010).unwrap();
        flash.wait_write().unwrap();
        flash.assert_info();

        //setup read parameters, no wrap, default strength, default burst, 8 dummy cycles
        //pg 19
        flash.enable_write().unwrap();
        flash.write_reg(0xC0, 0b1111_1000).unwrap();
        flash.wait_write().unwrap();

        flash
    }

    /// Erase all or some of the chip.
    ///
    /// Remarks:
    /// - Erasing sets all the bits in the given area to `1`.
    /// - The memory array of the IS25LP064A/032A is organized into uniform 4 Kbyte sectors or
    /// 32/64 Kbyte uniform blocks (a block consists of eight/sixteen adjacent sectors
    /// respectively).
    pub fn erase(&mut self, op: FlashErase) -> NBFlashResult<()> {
        match self.state {
            FlashState::Erasing(e) => {
                assert_eq!(e, op);
                if self.write_complete()? {
                    self.state = FlashState::Idle;
                    Ok(())
                } else {
                    Err(nbError::WouldBlock)
                }
            }
            FlashState::Idle => {
                self.enable_write()?;
                self.wait();
                match op {
                    FlashErase::Chip => self.write_command(0x60),
                    FlashErase::Sector4K(a) => self.qspi.write_extended(
                        QspiWord::U8(0xD7),
                        QspiWord::U24(a as _),
                        QspiWord::None,
                        &[],
                    ),
                    FlashErase::Block32K(a) => self.qspi.write_extended(
                        QspiWord::U8(0x52),
                        QspiWord::U24(a as _),
                        QspiWord::None,
                        &[],
                    ),
                    FlashErase::Block64K(a) => self.qspi.write_extended(
                        QspiWord::U8(0xD8),
                        QspiWord::U24(a as _),
                        QspiWord::None,
                        &[],
                    ),
                }?;
                self.state = FlashState::Erasing(op);
                Err(nbError::WouldBlock)
            }
            _ => panic!("not idle or erasing"),
        }
    }

    /// Read `data` out of the flash starting at the given `address`
    pub fn read(&mut self, address: u32, data: &mut [u8]) -> FlashResult<()> {
        assert_eq!(self.state, FlashState::Idle);
        let mut addr = address;
        //see page 34 for allowing to skip instruction
        assert!((addr as usize + data.len()) < 0x800000);
        for chunk in data.chunks_mut(32) {
            self.wait();
            self.qspi.read_extended(
                QspiWord::U8(0xEB),
                QspiWord::U24(addr),
                QspiWord::U8(0x00), //only A in top byte does anything
                8,
                chunk,
            )?;
            addr += 32;
        }
        Ok(())
    }

    /// Program `data` into the flash starting at the given `address`
    ///
    /// Remarks:
    /// - This operation can only set 1s to 0s, you must use `erase` to set a 0 to a 1.
    /// - The starting byte can be anywhere within the page (256 byte chunk). When the end of the
    /// page is reached, the address will wrap around to the beginning of the same page. If the
    /// data to be programmed are less than a full page, the data of all other bytes on the same
    /// page will remain unchanged.
    pub fn program(&mut self, address: u32, data: &[u8]) -> NBFlashResult<()> {
        let prog = |flash: &mut Self, chunk_index: u32| -> NBFlashResult<()> {
            if let Some(chunk) = data.chunks(32).nth(chunk_index as usize) {
                flash.enable_write()?;
                flash.wait();
                flash.qspi.write_extended(
                    QspiWord::U8(0x02),
                    QspiWord::U24(address + chunk_index * 32),
                    QspiWord::None,
                    chunk,
                )?;
                flash.state = FlashState::Programming {
                    address,
                    chunk: chunk_index,
                };
                Err(nbError::WouldBlock)
            } else {
                flash.state = FlashState::Idle;
                Ok(())
            }
        };
        match self.state {
            FlashState::Idle => prog(self, 0),
            FlashState::Programming {
                address: addr,
                chunk,
            } => {
                assert_eq!(addr, address);
                if self.write_complete()? {
                    prog(self, chunk + 1)
                } else {
                    Err(nbError::WouldBlock)
                }
            }
            _ => panic!("invalid state for programming"),
        }
    }
}
