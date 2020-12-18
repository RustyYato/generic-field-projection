pub trait CallOnce<A> {
    type Output;

    fn call_once(self, arg: A) -> Self::Output;
}

pub trait CallMut<A>: CallOnce<A> {
    fn call_mut(&mut self, arg: A) -> Self::Output;
}

#[doc(hidden)]
#[macro_export]
macro_rules! call {
    (fn $([ $($generics:tt)* ])? ($self:ident: $name:ty $(, $args:ident: $args_ty:ty)*) -> $output:ty
    $(where
        $($bound:ty : ($($bound_const:tt)*)),* $(,)?)?
        $body:block) => {
        impl$(<$($generics)*>)? $crate::type_list::call::CallOnce<($($args_ty,)*)> for $name
        where
            $( $($bound: $($bound_const)*,)* )?
        {
            type Output = $output;

            #[allow(unused_mut)]
            fn call_once(mut $self, ($($args,)*): ($($args_ty,)*)) -> Self::Output {
                $body
            }
        }
    };
    (fn $([ $($generics:tt)* ])? (&mut $self:ident: $name:ty $(, $args:ident: $args_ty:ty)*) -> $output:ty
    $(where
        $($bound:ty : ($($bound_const:tt)*)),* $(,)?)?
        $body:block) => {
        impl$(<$($generics)*>)? $crate::type_list::call::CallOnce<($($args_ty,)*)> for $name
        where
            $( $($bound: $($bound_const)*,)* )?
        {
            type Output = $output;

            fn call_once(mut self, args: ($($args_ty,)*)) -> Self::Output {
                <Self as $crate::type_list::call::CallMut<_>>::call_mut(&mut self, args)
            }
        }

        impl$(<$($generics)*>)? $crate::type_list::call::CallMut<($($args_ty,)*)> for $name
        where
            $( $($bound: $($bound_const)*,)* )?
        {
            #[allow(unused_mut)]
            fn call_mut(&mut $self, ($($args,)*): ($($args_ty,)*)) -> Self::Output {
                $body
            }
        }
    };
}
