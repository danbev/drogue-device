use crate::bsp::Board;
use crate::drivers::button::Button;
use crate::drivers::led::{ActiveHigh, Led};
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{PA0, PC6, PC7, PC8, PC9, USART1};
use embassy_stm32::usart::{Config, Uart};

pub type PinLedRed = Output<'static, PC6>;
pub type LedRed = Led<PinLedRed, ActiveHigh>;

pub type PinLedBlue = Output<'static, PC7>;
pub type LedBlue = Led<PinLedBlue, ActiveHigh>;

pub type PinLedOrange = Output<'static, PC8>;
pub type LedOrange = Led<PinLedOrange, ActiveHigh>;

pub type PinLedGreen = Output<'static, PC9>;
pub type LedGreen = Led<PinLedGreen, ActiveHigh>;

pub type PinUserButton = Input<'static, PA0>;
pub type UserButton = Button<ExtiInput<'static, PA0>>;

pub struct Stm32f072bDisco<'a> {
    pub led_red: LedRed,
    pub led_blue: LedBlue,
    pub led_orange: LedOrange,
    pub led_green: LedGreen,
    pub user_button: UserButton,
    pub usart1: Option<Uart<'a, USART1>>,
}

#[derive(Copy, Clone)]
pub enum UartConfig {
    Uart1PortA,
    Uart1PortB,
}

#[derive(Copy, Clone)]
pub struct Stm32f072bDiscoConfig {
    pub uart_config: UartConfig,
}

impl Default for Stm32f072bDiscoConfig {
    fn default() -> Self {
        Self {
            uart_config: UartConfig::Uart1PortA,
        }
    }
}

impl Board for Stm32f072bDisco<'_> {
    type Peripherals = embassy_stm32::Peripherals;
    type Config = Stm32f072bDiscoConfig;

    fn new(p: Self::Peripherals, config: Option<Self::Config>) -> Self {
        let usart1 = match config {
            None => None,
            Some(board_config) => match board_config.uart_config {
                UartConfig::Uart1PortA => Some(Uart::new(
                    p.USART1,
                    p.PA10,
                    p.PA9,
                    NoDma,
                    NoDma,
                    Config::default(),
                )),
                UartConfig::Uart1PortB => Some(Uart::new(
                    p.USART1,
                    p.PB7,
                    p.PB6,
                    NoDma,
                    NoDma,
                    Config::default(),
                )),
            },
        };

        let led_red = Led::new(Output::new(p.PC6, Level::High, Speed::Low));
        let led_blue = Led::new(Output::new(p.PC7, Level::High, Speed::Low));
        let led_orange = Led::new(Output::new(p.PC8, Level::High, Speed::Low));
        let led_green = Led::new(Output::new(p.PC9, Level::High, Speed::Low));
        let button = Input::new(p.PA0, Pull::Up);
        let user_button = Button::new(ExtiInput::new(button, p.EXTI0));

        Self {
            led_red,
            led_blue,
            led_orange,
            led_green,
            user_button,
            usart1,
        }
    }
}

impl Stm32f072bDisco<'_> {
    pub fn uart_description<'a>(uart_config: &UartConfig) -> &'a [u8] {
        let desc = match *uart_config {
            UartConfig::Uart1PortA => b"UART1 Tx: PA9, Rx: PA10\r\n",
            UartConfig::Uart1PortB => b"UART1 Tx: PB6, Rx: PB7 \r\n",
        };
        desc
    }
}
