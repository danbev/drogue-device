use crate::bsp::Board;
use crate::drivers::button::Button;
use crate::drivers::led::{ActiveHigh, Led};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{PC6, PC7, PC8, PC9, PA0};

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

pub struct Stm32f072bDisco {
    pub led_red: LedRed,
    pub led_blue: LedBlue,
    pub led_orange: LedOrange,
    pub led_green: LedGreen,
    pub user_button: UserButton,
}

impl Stm32f072bDisco {
    pub fn config() -> embassy_stm32::Config {
        let config = embassy_stm32::Config::default();
        config
    }
}

impl Board for Stm32f072bDisco {
    type Peripherals = embassy_stm32::Peripherals;
    fn new(p: Self::Peripherals) -> Self {
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
        }
    }
}
