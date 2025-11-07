//! # AnyTrait
//!
//! **AnyTrait** lets you upcast to a generic `&dyn AnyTrait` like
//! [`::core::any::Any`]\
//! but instead of just allowing you to downcast back to
//! the concrete type, it also lets you downcast to any trait used by your type
//!
//! *This is not zero-cost, since at any cast we need to go through the
//! list of all possible subtraits.*
//!
//! This will (almost) enable you to do OOP in rust, but if this is your goal
//! we still ask you to kindly reconsider
//!
//! example usage:
//! ```rust
//! use any_trait::{AnySubTrait, AnyTrait, AsAnyTrait};
//! trait TA {}
//! trait TB : AnyTrait {} // if a trait implements `AnyTrait` you can upcast
//! #[derive(AnySubTrait)]
//! #[any_sub_trait(TA, TB)] // must include all traits you want to downcast to
//! struct Concrete {
//!     // whatever
//! }
//! impl TA for Concrete {}
//! impl TB for Concrete {}
//! fn test() {
//!     let c = Concrete{};
//!     let a = c.as_anytrait();
//!
//!     let ta :&dyn TA = a.downcast_ref::<dyn TA>().unwrap();
//!     let tb :&dyn TB = a.downcast_ref::<dyn TB>().unwrap();
//!
//!     let a2 = tb.as_anytrait();
//!     let c_ref : &Concrete = a2.downcast_ref::<Concrete>().unwrap();
//! }
//! ```

#![feature(const_type_name)]
#![no_std]
pub mod typeidconst;

use typeidconst::TypeIdConst;

pub use ::any_trait_macro::AnySubTrait;

/// # AnyTrait
///
/// **Don't implement manually**
///
/// use `#[derive(AnySubTrait)]`
///
/// <br/>
///
/// Imagine a Concrete type and all its subtraits\
/// AnyTrait lets you walk up and down the traits safely
///
/// With `::core::any::Any` you can only cast between the concrete type
/// and `Any`.\
/// With `AnyTrait` you can do that, plus any other trait in the middle
///
/// *`AnyTrait` is not necessarily fast as it needs check and track
/// the list of traits you are allowed to cast to.*
pub trait AnyTrait: 'static {
    /// returns a list of all possible traits that you can up/downcast to\
    /// This list always has at least two elements:
    /// * id 0: `TypeIdConst::of::<dyn AnyType>`
    /// * id 1: `TypeIdConst::of::<YourConcreteType>`
    ///
    /// Only the rest of the list (aka: from index 2) is ordered,
    /// so we can run a binary search there if there are many types.
    fn type_ids(&self) -> &'static [TypeIdConst];

    /// **very unsafe. don't use. internal only. Horror here. go away.**
    ///
    /// cast `self` to a trait in the `.type_ids()` list.\
    /// the pointer to the ref to the type in the list
    /// is then converted to `usize`.
    ///
    /// panics if list length is exceeded.
    unsafe fn cast_to(&self, trait_num: usize) -> usize;
    /// **very unsafe. don't use. internal only. Horror here. go away.**
    ///
    /// cast `self` to a trait in the `.type_ids()` list.\
    /// the pointer to the ref to the type in the list
    /// is then converted to `usize`.
    ///
    /// panics if list length is exceeded.
    unsafe fn cast_to_mut(&mut self, trait_num: usize) -> usize;
}

/// upcast from the concrete type
/// (or from any other trait that implements `AnyTrait`) to `&dyn AnyTrait`
///
/// Automatically implemented on all types that implement `AnyTrait`
pub trait AsAnyTrait: AnyTrait {
    fn as_anytrait(&self) -> &dyn AnyTrait;
}

// everybody can have the same implementation as the `dyn Any` is always
// the first type in the list
impl<T: AnyTrait + ?Sized> AsAnyTrait for T {
    fn as_anytrait(&self) -> &dyn AnyTrait {
        unsafe {
            let raw = self.cast_to(0);
            union U {
                ptr: usize,
                a: *const *const dyn AnyTrait,
            }
            let tmp = U { ptr: raw };
            return &**tmp.a;
        }
    }
}

impl dyn AnyTrait {
    /// Search the list of possible traits.
    ///
    /// If `self` can be cast to the generic parameter,
    /// return the index of the type in the list
    #[inline]
    pub fn trait_idx<T: ?Sized + 'static>(&self) -> Option<usize> {
        let t = TypeIdConst::of::<T>();

        let all_traits = self.type_ids();
        if all_traits[0] == t {
            return Some(0);
        }
        if all_traits[1] == t {
            return Some(1);
        }
        let sub_traits = &all_traits[2..];

        // 128: carefully chosen completely at random
        if sub_traits.len() < 128 {
            match sub_traits.iter().enumerate().find(|x| *x.1 == t) {
                Some((idx, _)) => Some(2 + idx),
                None => None,
            }
        } else {
            match sub_traits.binary_search(&t) {
                Ok(idx) => Some(2 + idx),
                Err(_) => None,
            }
        }
    }

    /// Safe cast to reference to a generic type.
    ///
    /// Only return Some(...) if it is safe to do so.
    #[inline]
    pub fn downcast_ref<T: ?Sized + 'static>(&self) -> Option<&T> {
        let Some(idx) = self.trait_idx::<T>() else {
            return None;
        };
        #[allow(unsafe_code)]
        unsafe {
            #[allow(trivial_casts)]
            let ta = self.cast_to(idx);
            union UT<T: ?Sized> {
                a: usize,
                d: *const *const T,
            }
            let f = UT::<T> { a: ta };
            let res = *f.d;
            Some(&*res)
        }
    }

    /// Safe cast to mutable reference to a generic type.
    ///
    /// Only return Some(...) if it is safe to do so.
    #[inline]
    pub fn downcast_mut<T: ?Sized + 'static>(&mut self) -> Option<&mut T> {
        let Some(idx) = self.trait_idx::<T>() else {
            return None;
        };
        #[allow(unsafe_code)]
        unsafe {
            #[allow(trivial_casts)]
            let ta = self.cast_to_mut(idx);
            union UT<T: ?Sized> {
                a: usize,
                d: *mut *mut T,
            }
            let f = UT::<T> { a: ta };
            let res: *mut T = *f.d;
            Some(&mut *res)
        }
    }
}
