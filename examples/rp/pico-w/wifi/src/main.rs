#![no_std]
#![no_main]
#![feature(generic_associated_types, type_alias_impl_trait)]

use core::convert::Infallible;
use core::future::Future;

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Stack, StackResources};
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi::{ExclusiveDevice, SpiBusFlush, SpiBusRead, SpiBusWrite};
use embedded_io::asynch::{Read, Write};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use drogue_device::{ * };

const WIFI_SSID: &str = drogue::config!("wifi-ssid");
const WIFI_PSK: &str = drogue::config!("wifi-password");
const HOST: &str = drogue::config!("hostname");
const PORT: &str = drogue::config!("port");
const USERNAME: &str = drogue::config!("http-username");
const PASSWORD: &str = drogue::config!("http-password");
const LISTEN_PORT: u16 = 1234;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        ExclusiveDevice<MySpi, Output<'static, PIN_25>>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Drogue IoT Pico Pi W WiFi example");

    let p = embassy_rp::init(Default::default());

    // Include the WiFi firmware and Country Locale Matrix (CLM) blobs.
    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let clk = Output::new(p.PIN_29, Level::Low);
    let mut dio = Flex::new(p.PIN_24);
    dio.set_low();
    dio.set_as_output();

    let bus = MySpi { clk, dio };
    let spi = ExclusiveDevice::new(bus, cs);

    let state = singleton!(cyw43::State::new());
    let (mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    spawner.spawn(wifi_task(runner)).unwrap();

    let net_device = control.init(clm).await;

    control.join_wpa2(WIFI_SSID, WIFI_PSK).await;

    let dhcp_config = embassy_net::ConfigStrategy::Dhcp;

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let network = &*singleton!(Stack::new(
        net_device,
        dhcp_config,
        singleton!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(network)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(network, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        info!("Listening on TCP:${}...", LISTEN_PORT);
        if let Err(e) = socket.accept(LISTEN_PORT).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("Received {:02x}", &buf[..n]);
            info!("Sending {:02x} to drogue cloud", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

struct MySpi {
    /// SPI clock
    clk: Output<'static, PIN_29>,

    /// 4 signals, all in one!!
    /// - SPI MISO
    /// - SPI MOSI
    /// - IRQ
    /// - strap to set to gSPI mode on boot.
    dio: Flex<'static, PIN_24>,
}

impl ErrorType for MySpi {
    type Error = Infallible;
}

impl SpiBusFlush for MySpi {
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        async move { Ok(()) }
    }
}

impl SpiBusRead<u32> for MySpi {
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, words: &'a mut [u32]) -> Self::ReadFuture<'a> {
        async move {
            self.dio.set_as_input();
            for word in words {
                let mut w = 0;
                for _ in 0..32 {
                    w = w << 1;

                    // rising edge, sample data
                    if self.dio.is_high() {
                        w |= 0x01;
                    }
                    self.clk.set_high();

                    // falling edge
                    self.clk.set_low();
                }
                *word = w
            }

            Ok(())
        }
    }
}

impl SpiBusWrite<u32> for MySpi {
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, words: &'a [u32]) -> Self::WriteFuture<'a> {
        async move {
            self.dio.set_as_output();
            for word in words {
                let mut word = *word;
                for _ in 0..32 {
                    // falling edge, setup data
                    self.clk.set_low();
                    if word & 0x8000_0000 == 0 {
                        self.dio.set_low();
                    } else {
                        self.dio.set_high();
                    }

                    // rising edge
                    self.clk.set_high();

                    word = word << 1;
                }
            }
            self.clk.set_low();

            self.dio.set_as_input();
            Ok(())
        }
    }
}
