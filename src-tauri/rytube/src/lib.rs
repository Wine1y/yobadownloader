pub mod structs;
pub use structs::{Video, Thumbnail, Error, Stream};

mod parser;
mod deserializers;
mod client_builder;
mod decryption;
use structs::error::unwrap_or_return_error;

trait TryCollect<T>: Iterator {
    fn try_collect(self) -> Option<T>;
    fn try_collect_lossy(self) -> Option<T> where Self: Sized { None }
}

impl<T> TryCollect<(T::Item, )> for T
    where T: Iterator {
    #[inline]
    fn try_collect(mut self) -> Option<(T::Item, )> {
        match (self.next(), self.next()) {
            (Some(item), None) => Some((item, )),
            _ => None
        }
    }

    #[inline]
    fn try_collect_lossy(mut self) -> Option<(T::Item, )> {
        self.next().map(|v| (v, ))
    }
}

impl<T> TryCollect<(T::Item, T::Item)> for T
    where T: Iterator {
    #[inline]
    fn try_collect(mut self) -> Option<(T::Item, T::Item)> {
        match (self.next(), self.next(), self.next()) {
            (Some(item1), Some(item2), None) => Some((item1, item2)),
            _ => None
        }
    }

    #[inline]
    fn try_collect_lossy(mut self) -> Option<(T::Item, T::Item)> {
        match (self.next(), self.next()) {
            (Some(item1), Some(item2)) => Some((item1, item2)),
            _ => None
        }
    }
}

