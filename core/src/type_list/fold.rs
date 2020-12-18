use super::*;

pub trait ListFold<A, F> {
    type Output;

    fn list_fold(self, acc: A, f: F) -> Self::Output;
}

impl<A, F> ListFold<A, F> for Nil {
    type Output = A;

    fn list_fold(self, acc: A, _: F) -> Self::Output {
        acc
    }
}

impl<T, A, F> ListFold<A, F> for Cons<T, Nil>
where
    F: CallOnce<(A, T)>,
{
    type Output = F::Output;

    fn list_fold(self, acc: A, f: F) -> Self::Output {
        f.call_once((acc, self.0))
    }
}

impl<T, A, F, U, R> ListFold<A, F> for Cons<T, Cons<U, R>>
where
    F: CallMut<(A, T)>,
    Cons<U, R>: ListFold<F::Output, F>,
{
    type Output = <Cons<U, R> as ListFold<F::Output, F>>::Output;

    fn list_fold(self, acc: A, mut f: F) -> Self::Output {
        let f_out = f.call_mut((acc, self.0));
        self.1.list_fold(f_out, f)
    }
}
