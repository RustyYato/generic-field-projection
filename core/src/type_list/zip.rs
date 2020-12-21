use super::{Cons, Nil};

pub type Zipped<A, B> = <A as Zip<B>>::Output;

pub trait Zip<L> {
    type Output;
    fn zip(self, other: L) -> Self::Output;
}

impl Zip<Nil> for Nil {
    type Output = Nil;

    fn zip(self, Nil: Self) -> Self::Output {
        Self
    }
}

impl<A, B, Ra: Zip<Rb>, Rb> Zip<Cons<B, Rb>> for Cons<A, Ra> {
    type Output = Cons<(A, B), Zipped<Ra, Rb>>;

    fn zip(self, other: Cons<B, Rb>) -> Self::Output {
        Cons((self.0, other.0), self.1.zip(other.1))
    }
}
