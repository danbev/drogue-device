#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use embassy_stm32::{Peripherals};
use embedded_hal::blocking::serial::Write;
use drogue_device::{bind_bsp, Board,};
use drogue_device::bsp::boards::stm32f0::stm32f072b_disco::{
    Stm32f072bDisco, Stm32f072bDiscoConfig as BoardConfig,
    UartConfig::{Uart1PortB}
};
use panic_probe as _;
use defmt_rtt as _;

bind_bsp!(Stm32f072bDisco<'static>, BSP);

fn config() -> embassy_stm32::Config {
    let config = embassy_stm32::Config::default();
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: embassy::executor::Spawner, p: Peripherals) {
    let board_config = BoardConfig { uart_config: Uart1PortB};
    let mut usart1 = BSP::new(p, Some(board_config)).0.usart1.unwrap();

    usart1.bwrite_all(b"STM32F072B Discovery Board UART Example\r\n").unwrap();
    usart1.bwrite_all(Stm32f072bDisco::uart_description(&board_config.uart_config)).unwrap();
}
