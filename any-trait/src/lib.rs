//#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![no_std]

//! # AnyTrait
//!
//! This is a **no_std** crate that lets you cast from:
//! * your concrete type
//! * `&dyn AnyTrait`
//! to:
//! * the concrete type
//! * any other trait implemented by your type
//! * `&dyn AnyTrait`
//!
//! If the trait implements `AnyTrait`, that too can be cast to:
//! * the concrete type
//! * any other trait implemented by your type
//! * `&dyn AnyTrait`
//!
//! *This is not zero-cost, since at any cast we need to go through the
//! list of all possible subtraits.*
//!
//! This will (almost) enable you to do OOP in rust, but if this is your goal
//! we still ask you to kindly reconsider
//!
//! example usage:
//! ```rust
//! use any_trait::{AnySubTrait, AnyTrait, AsAnyTrait, AnyTraitCast};
//! trait TA {}
//! trait TB : AnyTrait {} // if a trait implements `AnyTrait` you can upi/downcast
//! #[derive(AnySubTrait)]
//! #[any_sub_trait(TA, TB)] // must include all traits you want to downcast to
//! struct Concrete {
//!     // whatever
//! }
//! impl TA for Concrete {}
//! impl TB for Concrete {}
//! fn test() {
//!     let c = Concrete{};
//!
//!     let a = c.as_anytrait();
//!
//!     let ta :&dyn TA = a.cast_ref::<dyn TA>().unwrap();
//!     let tb :&dyn TB = a.cast_ref::<dyn TB>().unwrap();
//!
//!     let ta_from_tb : &dyn TA = tb.cast_ref::<dyn TA>().unwrap();
//!
//!     let a2 = tb.as_anytrait();
//!     let c_ref : &Concrete = a2.cast_ref::<Concrete>().unwrap();
//! }
//! ```
pub mod anyptr;
pub mod typeidconst;

use anyptr::AnyPtr;
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
    /// The reset of the list is currently unordered, will change as soon
    /// as we find a way to have a `const Ord` on `TypeId`
    fn type_ids(&self) -> &'static [TypeIdConst];

    /// **don't use. internal only.**
    ///
    /// cast `self` to a trait in the `.type_ids()` list.\
    /// the pointer to the ref to the type in the list
    /// is then type-erase to `AnyPtr`.
    ///
    /// # Safety
    /// This is safe since you can't do anything to a `AnyPtr` by itself.
    /// `AnyPtr::to_ptr` however is just wrong if you don't have the right
    /// type. Again, don't use: internal only
    ///
    /// # Panics
    /// If list `trait_num` exceeds `type_ids()` length
    fn type_erase(&self, trait_num: usize) -> AnyPtr;
    /// **don't use. internal only.**
    ///
    /// cast `self` to a trait in the `.type_ids()` list.\
    /// the pointer to the ref to the type in the list
    /// is then type-erase to `AnyPtr`.
    ///
    /// # Safety
    /// This is safe since you can't do anything to a `AnyPtr` by itself
    /// `AnyPtr::to_ptr` however is just wrong if you don't have the right
    /// type. Again, don't use: internal only
    ///
    /// # Panics
    /// If list `trait_num` exceeds `type_ids()` length
    fn type_erase_mut(&mut self, trait_num: usize) -> AnyPtr;
}

/// upcast from the concrete type
///
/// (or from any other trait that implements `AnyTrait`) to `&dyn AnyTrait`
///
/// **Automatically implemented on all types that implement `AnyTrait`**
pub trait AsAnyTrait: AnyTrait {
    fn as_anytrait(&self) -> &dyn AnyTrait;
    fn as_anytrait_mut(&mut self) -> &mut dyn AnyTrait;
}

/// (Up/Down)cast to another type
///
/// **Automatically implemented on all types that implement `AnyTrait`**
///
/// Note that this trait is **not dyn-compatible** and for that reason it is
/// kept separate
pub trait AnyTraitCast: AnyTrait {
    /// Find the type in the supported trait list
    fn trait_idx<T: ?Sized + 'static>(&self) -> Option<usize>;
    /// (Up/Down)cast to a ref if the type is supported.
    ///
    /// Both Upcast and Downcast work, as long as the type is supported
    fn cast_ref<D: ?Sized + 'static>(&self) -> Option<&D>;
    /// (Up/Down)cast to a mut ref if the type is supported.
    ///
    /// Both Upcast and Downcast work, as long as the type is supported
    fn cast_mut<D: ?Sized + 'static>(&mut self) -> Option<&mut D>;
}

// everybody can have the same implementation as the `dyn Any` is always
// the first type in the list
impl<T: AnyTrait + ?Sized> AsAnyTrait for T {
    /// upcast to `&dyn AnyTrait`
    #[inline]
    fn as_anytrait(&self) -> &dyn AnyTrait {
        let erased = self.type_erase(0);
        #[allow(unsafe_code)]
        unsafe {
            let any = erased.to_ptr::<dyn AnyTrait>();

            return any.as_ref();
        }
    }
    /// upcast to `&mut dyn AnyTrait`
    fn as_anytrait_mut(&mut self) -> &mut dyn AnyTrait {
        let erased = self.type_erase(0);
        #[allow(unsafe_code)]
        unsafe {
            let mut any = erased.to_ptr::<dyn AnyTrait>();

            return any.as_mut();
        }
    }
}

impl<T: AnyTrait + ?Sized> AnyTraitCast for T {
    /// Search the list of possible traits.
    ///
    /// If `self` can be cast to the generic parameter,
    /// return the index of the type in the list
    #[inline]
    fn trait_idx<D: ?Sized + 'static>(&self) -> Option<usize> {
        let t = TypeIdConst::of::<D>();

        let all_traits = self.type_ids();

        for it in all_traits.iter().enumerate() {
            if *it.1 == t {
                return Some(it.0);
            }
        }

        return None;

        /* Waiting for const Ord on TypeId...
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
        */
    }

    /// Safe cast to reference to a generic type.
    ///
    /// Only return Some(...) if it is safe to do so.
    #[inline]
    fn cast_ref<D: ?Sized + 'static>(&self) -> Option<&D> {
        let Some(trait_idx) = self.trait_idx::<D>() else {
            return None;
        };

        let erased = self.type_erase(trait_idx);
        #[allow(unsafe_code)]
        unsafe {
            let any = erased.to_ptr::<D>();

            return Some(any.as_ref());
        }
    }

    /// Safe cast to mutable reference to a generic type.
    ///
    /// Only return Some(...) if it is safe to do so.
    #[inline]
    fn cast_mut<D: ?Sized + 'static>(&mut self) -> Option<&mut D> {
        let Some(trait_idx) = self.trait_idx::<D>() else {
            return None;
        };

        let erased = self.type_erase_mut(trait_idx);
        #[allow(unsafe_code)]
        unsafe {
            let mut any = erased.to_ptr::<D>();

            return Some(any.as_mut());
        }
    }
}
