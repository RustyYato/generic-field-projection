use super::*;

pub type Mapped<L, F> = <L as ListMap<F>>::Output;
pub trait ListMap<F> {
    type Output;

    fn map(self, f: F) -> Self::Output;
}

impl<F> ListMap<F> for Nil {
    type Output = Nil;

    fn map(self, _: F) -> Self::Output {
        Nil
    }
}

impl<T, F> ListMap<F> for Cons<T, Nil>
where
    F: CallOnce<(T,)>,
{
    type Output = Cons<F::Output, Nil>;

    fn map(self, f: F) -> Self::Output {
        Cons(f.call_once((self.0,)), Nil)
    }
}

impl<T, F, U, R> ListMap<F> for Cons<T, Cons<U, R>>
where
    F: CallMut<(T,)>,
    Cons<U, R>: ListMap<F>,
{
    type Output = Cons<F::Output, <Cons<U, R> as ListMap<F>>::Output>;

    fn map(self, mut f: F) -> Self::Output {
        Cons(f.call_mut((self.0,)), self.1.map(f))
    }
}
