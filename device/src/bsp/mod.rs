//! Board Support Packages (BSP).

pub mod boards;

/// A board capable of creating itself using peripherals.
pub trait Board: Sized {
    type Peripherals;
    type Config;

    fn new(peripherals: Self::Peripherals, config: Option<Self::Config>) -> Self;
}

#[macro_export]
macro_rules! bind_bsp {
    ($bsp:ty, $app_bsp:ident) => {
        struct $app_bsp($bsp);
        impl $crate::bsp::Board for BSP {
            type Peripherals = <$bsp as $crate::bsp::Board>::Peripherals;
            type Config = <$bsp as $crate::bsp::Board>::Config;

            fn new(peripherals: Self::Peripherals, config: Option<Self::Config>) -> Self {
                BSP(<$bsp>::new(peripherals, config))
            }
        }
    };
}
