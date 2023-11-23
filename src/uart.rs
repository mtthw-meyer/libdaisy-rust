use stm32h7xx_hal::{
    self as hal, rcc, serial::config::Config, serial::Serial, serial::SerialExt, stm32,
};

pub struct UART {
    pub usart1: Option<(stm32::USART1, rcc::rec::Usart1)>,
    pub usart3: Option<(stm32::USART3, rcc::rec::Usart3)>,
    pub uart4: Option<(stm32::UART4, rcc::rec::Uart4)>,
    pub uart5: Option<(stm32::UART5, rcc::rec::Uart5)>,
}

pub fn serial<
    UART,
    UARTSERIAL: SerialExt<UART>,
    TX: stm32h7xx_hal::serial::PinTx<UART>,
    RX: stm32h7xx_hal::serial::PinRx<UART>,
>(
    tx: TX,
    rx: RX,
    config: Config,
    uart: (UARTSERIAL, UARTSERIAL::Rec),
    clocks: &hal::rcc::CoreClocks,
) -> Serial<UART> {
    uart.0.serial((tx, rx), config, uart.1, clocks).unwrap()
}
