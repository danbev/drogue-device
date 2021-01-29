use crate::actor::Actor;
use crate::address::Address;
use crate::bus::EventBus;
use crate::device::{Device, Lifecycle};
use crate::handler::{Completion, NotificationHandler, RequestHandler, Response};
use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use heapless::{consts::*, spsc::Queue};

pub struct Lock;

pub struct Unlock<T>(T);

pub struct Mutex<T>
    where
        T: 'static
{
    address: Option<Address<Self>>,
    val: Option<T>,
    waiters: Queue<Waker, U16>,
}

impl<T> Actor for Mutex<T>
    where
        T: 'static
{
    fn mount(&mut self, addr: Address<Self>) {
        self.address.replace(addr);
    }
}

impl<T> RequestHandler<Lock>
for Mutex<T>
    where
        T: 'static
{
    type Response = Exclusive<T>;

    fn on_request(&'static mut self, message: Lock) -> Response<Self::Response> {
        Response::defer(async move {
            let lock = Exclusive {
                address: self.address.as_ref().unwrap().clone(),
                val: Some(self.lock().await),
            };
            log::trace!("[Mutex<T> lock");
            lock
        })
    }
}

impl<T> NotificationHandler<Lifecycle>
for Mutex<T>
    where
        T: 'static
{
    fn on_notification(&'static mut self, message: Lifecycle) -> Completion {
        Completion::immediate()
    }
}

impl<T> NotificationHandler<Unlock<T>>
for Mutex<T>
    where
        T: 'static
{
    fn on_notification(&'static mut self, message: Unlock<T>) -> Completion {
        log::trace!("[Mutex<T> unlock");
        self.unlock(message.0);
        Completion::immediate()
    }
}

impl<T> Mutex<T>
{
    pub fn new(val: T) -> Self {
        Self {
            address: None,
            val: Some(val),
            waiters: Queue::new(),
        }
    }

    pub async fn lock(&'static mut self) -> T {
        struct LockFuture<TT: 'static> {
            waiting: bool,
            mutex: UnsafeCell<*mut Mutex<TT>>,
        }

        impl<TT> Future for LockFuture<TT> {
            type Output = TT;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe {
                    if let Some(val) = (**self.mutex.get()).val.take() {
                        Poll::Ready(val)
                    } else {
                        if !self.waiting {
                            self.waiting = true;
                            (**self.mutex.get())
                                .waiters
                                .enqueue(cx.waker().clone())
                                .unwrap_or_else(|_| panic!("too many waiters"))
                        }
                        Poll::Pending
                    }
                }
            }
        }

        LockFuture {
            waiting: false,
            mutex: UnsafeCell::new(self),
        }
            .await
    }

    pub fn unlock(&mut self, val: T) {
        self.val.replace(val);
        if let Some(next) = self.waiters.dequeue() {
            next.wake()
        }
    }
}

pub struct Exclusive<T>
    where
        T: 'static
{
    val: Option<T>,
    address: Address<Mutex<T>>,
}

impl<T> Deref
for Exclusive<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val.as_ref().unwrap()
    }
}

impl<T> DerefMut
for Exclusive<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val.as_mut().unwrap()
    }
}

impl<T> Drop for Exclusive<T>
    where
        T: 'static
{
    fn drop(&mut self) {
        self.address.notify(Unlock(self.val.take().unwrap()))
    }
}

impl<T> Address<Mutex<T>>
{
    pub async fn lock(&'static self) -> Exclusive<T> {
        self.request(Lock).await
    }
}
