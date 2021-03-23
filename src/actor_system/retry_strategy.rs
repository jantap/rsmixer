use crate::prelude::*;

use std::{marker::PhantomData, pin::Pin};

use futures::Future;

pub trait StrategyClosure: Future<Output = bool> + Send + Sync {}
impl<X> StrategyClosure for X where X: Future<Output = bool> + Send + Sync {}

pub type PinnedClosure = Pin<Box<dyn StrategyClosure>>;

pub trait Strategy<T>: Fn(T) -> PinnedClosure + Send + Sync {}
impl<X, T> Strategy<T> for X where X: Fn(T) -> PinnedClosure + Send + Sync {}


pub struct RetryStrategy {
    pub on_panic: Box<dyn Strategy<usize>>,
    pub on_error: Box<dyn Strategy<(usize, Result<()>)>>,
    pub retry_count: usize,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            on_panic: Box::new(|_: usize| Box::pin(async { false })),
            on_error: Box::new(|(_, _)| Box::pin(async { false })),
            retry_count: 0,
        }
    }
}
