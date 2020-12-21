use super::*;
use fold::ListFold;

pub trait ListAny<F> {
    fn any(self, f: F) -> bool;
}

pub struct AnyFolder<F>(F);

impl<F, L> ListAny<F> for L
where
    Self: ListFold<bool, AnyFolder<F>, Output = bool>,
{
    fn any(self, f: F) -> bool {
        self.fold(false, AnyFolder(f))
    }
}

impl<T, F> CallOnce<(bool, T)> for AnyFolder<F>
where
    F: CallOnce<(T,), Output = bool>,
{
    type Output = bool;

    fn call_once(self, (acc, arg): (bool, T)) -> Self::Output {
        acc || self.0.call_once((arg,))
    }
}

impl<T, F> CallMut<(bool, T)> for AnyFolder<F>
where
    F: CallMut<(T,), Output = bool>,
{
    fn call_mut(&mut self, (acc, arg): (bool, T)) -> Self::Output {
        acc || self.0.call_mut((arg,))
    }
}
