use super::*;

pub struct PtrToRefMut<'a>(PhantomData<&'a ()>);

impl<'a> PtrToRefMut<'a> {
    #[inline]
    pub(crate) unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

type_function! {
    for('a, T: 'a + ?Sized)
    fn(self: PtrToRefMut<'a>, ptr: *mut T) -> &'a mut T {
        unsafe { &mut *ptr }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for &mut F {}
impl<'a, F: Field> ProjectTo<F> for &'a mut F::Parent
where F::Parent: 'a, F::Type: 'a {
    type Projection = &'a mut F::Type;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            &mut *field.project_raw_mut(self)
        }
    }
}

pub struct FindOverlap<S> {
    counter: u64,
    set: S
}

pub struct FindOverlapInner<I> {
    id: u64,
    counter: u64,
    field: I
}

type_function! {
    for(S: Copy + TupleAny<FindOverlapInner<I>>, I: Field)
    fn(self: FindOverlap<S>, input: I) -> bool {
        self.counter += 1;
        self.set.tup_any(FindOverlapInner {
            id: self.counter,
            counter: 0,
            field: input
        })
    }
    
    for(I: Field, J: Field)
    fn(self: FindOverlapInner<I>, input: J) -> bool {
        self.counter += 1;

        if self.id <= self.counter {
            return false
        }

        self.field.name().zip(input.name())
            .all(|(i, j)| i == j)
    }
}

impl<'a, F: FieldSet> ProjectToSet<F> for &'a mut F::Parent
where F::Parent: 'a,
      F::TypeSetMut: TupleMap<PtrToRefMut<'a>>,
      
      F: Copy + TupleAny<FindOverlap<F>> {
    type Projection = TMap<F::TypeSetMut, PtrToRefMut<'a>>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            if field.tup_any(FindOverlap {
                    counter: 0,
                    set: field
                }) {
                panic!("Found overlapping fields")
            } else {
                let type_set = field.project_raw_mut(self);
                type_set.tup_map(PtrToRefMut::new())
            }
        }
    }
}
