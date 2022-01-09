#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use bsp_blinky_app::{BlinkyBoard, BlinkyConfiguration, BlinkyDevice};
use drogue_device::{bind_bsp, Board, DeviceContext};
use embassy_stm32::Peripherals;
use drogue_device::bsp::boards::stm32f0::stm32f072b_disco::{
    Stm32f072bDisco,
    LedBlue,
    UserButton,
};
use panic_probe as _;
use defmt_rtt as _;

// Creates a newtype named `BSP` around the `Stm32f072b` to avoid
// orphan rules and apply delegation boilerplate.
bind_bsp!(Stm32f072bDisco, BSP);

/// Define the required associated types for easy reference to avoid
/// generic explosion for the details of this board to the app.
impl BlinkyBoard for BSP {
    type Led = LedBlue;
    type ControlButton = UserButton;
}

static DEVICE: DeviceContext<BlinkyDevice<BSP>> = DeviceContext::new();

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, p: Peripherals) {
    let board = BSP::new(p);

    let config = BlinkyConfiguration {
        led: board.0.led_blue,
        control_button: board.0.user_button,
    };

    DEVICE.configure(BlinkyDevice::new()).mount(spawner, config).await;
}
