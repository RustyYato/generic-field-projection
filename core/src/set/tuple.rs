
pub type FuncOut<F, I> = <F as TypeFunction<I>>::Output;

pub trait TypeFunction<Input> {
    type Output;

    fn call(&mut self, input: Input) -> Self::Output;
}

pub trait Tuple {}

pub type TMap<T, F> = <T as TupleMap<F>>::Output;

pub trait TupleMap<F>: Tuple + Sized {
    type Output;

    fn tup_map(self, f: F) -> Self::Output;
}

pub trait TupleAny<F>: Tuple + Sized {
    fn tup_any(self, f: F) -> bool;
}

macro_rules! impl_tuple {
    () => {
        impl Tuple for () {}

        impl<Func> TupleMap<Func> for () {
            type Output = ();

            fn tup_map(self, _: Func) -> Self::Output {}
        }
        
        impl<Func> TupleAny<Func> for () {
            fn tup_any(self, _: Func) -> bool {
                false
            }
        }
    };
    ($($T:ident)+) => {
        impl_tuple! { @rem $($T)* }
        
        #[allow(non_snake_case)]
        impl<$($T,)+> Tuple for ($($T,)+) {}

        #[allow(non_snake_case)]
        impl<Func $(, $T)+> TupleMap<Func> for ($($T,)+)
        where $(Func: TypeFunction<$T>),+ {
            type Output = ($(FuncOut<Func, $T>,)*);

            fn tup_map(self, mut func: Func) -> Self::Output {
                let ($($T,)+) = self;

                ($(
                    func.call($T),
                )+)
            }
        }
        
        #[allow(non_snake_case)]
        impl<Func $(, $T)+> TupleAny<Func> for ($($T,)+)
        where $(Func: TypeFunction<$T, Output = bool>),+  {
            fn tup_any(self, mut func: Func) -> bool {
                let ($($T,)+) = self;

                $(
                    if func.call($T) {
                        return true;
                    }
                )+
                
                false
            }
        }
    };
    (@rem $drop:ident $($T:ident)*) => {
        impl_tuple! { $($T)* }
    };
}

impl_tuple! {
    A B C D
    E F G H
    I J K L
    M N O P
}
