use super::*;

mod from_pin;
mod from_mut;
mod from_ref;

use core::pin::Pin;
use core::ops::Deref;
use core::marker::PhantomData;

use crate::pin::*;

use crate::set::tuple::TypeFunction;
