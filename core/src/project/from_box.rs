use super::*;

use crate::alloc::Box;
use std::ops::DerefMut;
use std::ptr::NonNull;

pub struct PtrToNonNull;

type_function! {
    for(T: ?Sized)
    fn(self: PtrToNonNull, ptr: *mut T) -> NonNull<T> { unsafe { NonNull::new_unchecked(ptr) } }
}

pub struct BoxProjection<T: ?Sized, F: ?Sized> {
    bx: NonNull<T>,
    field: NonNull<F>,
}

impl<T: ?Sized, F: ?Sized> Deref for BoxProjection<T, F> {
    type Target = F;

    fn deref(&self) -> &F {
        unsafe { self.field.as_ref() }
    }
}

impl<T: ?Sized, F: ?Sized> DerefMut for BoxProjection<T, F> {
    fn deref_mut(&mut self) -> &mut F {
        unsafe { self.field.as_mut() }
    }
}

unsafe impl<#[may_dangle] T: ?Sized, #[may_dangle] F: ?Sized> Drop for BoxProjection<T, F> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.bx.as_ptr());
        }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for Box<F> {}
impl<F: Field> ProjectTo<F> for Box<F::Parent> {
    type Projection = BoxProjection<F::Parent, F::Type>;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            let bx = Box::into_raw(self);
            let field = field.project_raw_mut(bx);
            let bx = NonNull::new_unchecked(bx);
            let field = NonNull::new_unchecked(field);

            BoxProjection { bx, field }
        }
    }
}

// TODO: figure out how to implement `ProjectToSet<F>` for `Box<F::Parent>`
// impl<'a, F: FieldSet> ProjectToSet<F> for Box<F::Parent>
// where F::Parent: 'a,
//       F::TypeSetMut: TupleMap<PtrToNonNull>,

//       F: Copy + TupleAny<FindOverlap<F>> {
//     type Projection = TMap<F::TypeSetMut, PtrToNonNull>;

//     #[inline]
//     fn project_set_to(self, field: F) -> Self::Projection {
//         unsafe {
//             if field.tup_any(FindOverlap::new(field)) {
//                 panic!("Found overlapping fields")
//             } else {
//                 let type_set = field.project_raw_mut(Box::into_raw(self));
//                 type_set.tup_map(PtrToNonNull)
//             }
//         }
//     }
// }
