use crate::Field;

use typsy::{
    call::Simple,
    hlist::{Cons, Nil},
    map::{Map, Mapped},
};

pub type Projected<Parent, F> = Mapped<F, Simple<ProjectRaw<Parent>>>;
pub type ProjectedMut<Parent, F> = Mapped<F, Simple<ProjectRawMut<Parent>>>;
pub struct ProjectRaw<Parent: ?Sized>(*const Parent);
pub struct ProjectRawMut<Parent: ?Sized>(*mut Parent);

impl<Parent: ?Sized> ProjectRaw<Parent> {
    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    pub unsafe fn new(parent: *const Parent) -> Self {
        Self(parent)
    }
}

impl<Parent: ?Sized> ProjectRawMut<Parent> {
    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    pub unsafe fn new(parent: *mut Parent) -> Self {
        Self(parent)
    }
}

typsy::call! {
    fn[F: Field](&self: ProjectRaw<F::Parent>, field: F) -> *const F::Type {
        unsafe { field.project_raw(self.0) }
    }

    fn[F: Field](&self: ProjectRawMut<F::Parent>, field: F) -> *mut F::Type {
        unsafe { field.project_raw_mut(self.0) }
    }
}

/// Represents a list of fields
pub trait FieldList<Parent: ?Sized>:
    Map<Simple<ProjectRaw<Parent>>> + Map<Simple<ProjectRawMut<Parent>>>
{
}

impl<Parent: ?Sized> FieldList<Parent> for Nil {
}

impl<F: Field, R> FieldList<F::Parent> for Cons<F, R> where
    Self: Map<Simple<ProjectRaw<F::Parent>>>
        + Map<Simple<ProjectRawMut<F::Parent>>>
{
}
