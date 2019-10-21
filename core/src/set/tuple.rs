#[macro_export]
macro_rules! type_function {
    (
        $($(for($($g_params:tt)*))? fn($self:ident:$func:ty $(, $param:ident: $type:ty )* $(,)? ) -> $output:ty $block:block)*
    ) => {$(
        #[allow(unused_parens)]
        impl $(<$($g_params)*>)? $crate::TypeFunction<($($type),*)> for $func {
            type Output = $output;

            #[inline]
            fn call(&mut $self, ($($param),*): ($($type),*)) -> Self::Output {
                $block
            }
        }
    )*};
}

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

pub type TZip<T, U, F> = <T as TupleZip<U, F>>::Output;

pub trait TupleZip<T, F>: Tuple + Sized {
    type Output;

    fn tup_zip(self, tuple: T, f: F) -> Self::Output;
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

macro_rules! impl_tuple_zip {
    () => {
        impl<Func> TupleZip<(), Func> for () {
            type Output = ();

            fn tup_zip(self, (): (), _: Func) -> Self::Output {}
        }
    };
    ($($T:ident $U:ident)+) => {
        impl_tuple_zip! { @rem $($T $U)* }

        #[allow(non_snake_case)]
        impl<Func $(, $T, $U)*> TupleZip<($($U,)*), Func> for ($($T,)*)
            where $(Func: TypeFunction<($T, $U)>),+
        {
            type Output = ($(
                <Func as TypeFunction<($T, $U)>>::Output,
            )*);

            fn tup_zip(self, ($($U,)*): ($($U,)*), mut func: Func) -> Self::Output {
                let ($($T,)*) = self;

                ($(
                    func.call(($T, $U)),
                )*)
            }
        }
    };
    (@rem $drop_0:ident $drop_1:ident $($T:ident)*) => {
        impl_tuple_zip! { $($T)* }
    };
}

impl_tuple! {
    A B C D
    E F G H
    I J K L
    M N O P
}

impl_tuple_zip! {
    A0 A1 B0 B1 C0 C1 D0 D1
    E0 E1 F0 F1 G0 G1 H0 H1
    I0 I1 J0 J1 K0 K1 L0 L1
    M0 M1 N0 N1 O0 O1 P0 P1
}
