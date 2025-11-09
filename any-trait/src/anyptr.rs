/*
 * This file is largely taken from `quinedot` response in:
 * https://users.rust-lang.org/t/cast-from-concrete-to-any-and-subtraits/136086/4?u=lucafulchir
 * and therefore the project's license and copyright don't apply here
 */

//! Type erasure for pointers
//!
//! Many, Many thanks to
//! [**quinedot**](https://users.rust-lang.org/t/cast-from-concrete-to-any-and-subtraits/136086/4?u=lucafulchir)
//! This is basically copied from his response

use ::core::{
    mem::{MaybeUninit, size_of, transmute_copy},
    ptr::NonNull,
};

/// A pointer to a `Sized` type or to a `dyn Trait`.
///
/// modified from:
/// [**quinedot**](https://users.rust-lang.org/t/cast-from-concrete-to-any-and-subtraits/136086/4?u=lucafulchir)
///
/// Rust has two main type of pointers:
/// * thin pointer: same size as `usize`. we store those directly in `.data`
/// * fat pointers: they have a `data` pointer and a `vtable` pointer.
///
/// Unfortunately in fat pointers there is no guerantee on which is first
/// (data or vtable), so we need to check every time.\
/// We store this information as a 1-byte offset in the `.meta` field.\
/// Vtable pointers are guaranteed to be word-aligned.
#[derive(Copy, Clone, Debug)]
pub struct AnyPtr {
    /// Pointer to the value.
    ///
    /// * If this is a thin pointer, `.meta` will be `None`
    /// * If this is a fat pointer, this is the `data` part of the fat pointer
    data: NonNull<()>,

    /// Pointer to a `dyn Trait` vtable.
    ///
    /// If this is `None`, `AnyPtr` is a thin pointer.
    /// If this is `Some()` then we have a fat pointer, and we have to
    /// check the alignment of the vtable.
    ///
    /// * vtable %2 == 0 => the vtable comse first, the data ptr second
    /// * vtable %2 == 1 => the data ptr comse first, the vtable second
    meta: Option<NonNull<()>>,
}

impl AnyPtr {
    /// Create a type-erased mutable pointer
    ///
    /// # Panics
    /// If `ptr` is Null.
    pub fn from<T: ?Sized>(ptr: *const T) -> Self {
        const {
            assert!(
                size_of::<*const T>() == size_of::<NonNull<()>>()
                    || size_of::<*const T>() == size_of::<[usize; 2]>()
            )
        }

        return Self::from_mut::<T>(ptr as *mut T);
    }

    /// Create a type-erased pointer
    ///
    /// # Panics
    /// If `ptr` is Null.
    pub fn from_mut<T: ?Sized>(ptr: *mut T) -> Self {
        const {
            assert!(
                size_of::<*const T>() == size_of::<NonNull<()>>()
                    || size_of::<*const T>() == size_of::<[*mut (); 2]>()
            )
        }
        if size_of::<*const T>() == size_of::<NonNull<()>>() {
            // THIN pointer
            return Self {
                data: NonNull::new(ptr).unwrap().cast(),
                meta: None,
            };
        }

        // FAT pointer
        let ptr = ptr as *const T;

        // Detect the data pointer by changing its address.
        // This will leave the metadata untouched.
        let ptr2 = ptr.wrapping_byte_add(1);

        // SAFETY: We've checked the size and are only comparing
        // plain-old-data values.
        let (before, after) = unsafe {
            (
                transmute_copy::<*const T, [*mut (); 2]>(&ptr),
                transmute_copy::<*const T, [*mut (); 2]>(&ptr2),
            )
        };
        let data = NonNull::new(ptr as *mut ()).unwrap();
        let meta = if before[0] == after[0] {
            // The data pointer should have been added to.
            debug_assert_eq!(before[1] as usize, after[1] as usize - 1);
            // Vtable pointers must be non-null and word-aligned.
            debug_assert_ne!(0, before[0] as usize);
            debug_assert_eq!(0, before[0] as usize % 2);

            // It was the first pointer so we store it directly.
            NonNull::new(before[0])
        } else if before[1] == after[1] {
            // The data pointer should have been added to.
            debug_assert_eq!(before[0] as usize, after[0] as usize - 1);
            // Vtable pointers must be non-null and word-aligned.
            debug_assert_ne!(0, before[1] as usize);
            debug_assert_eq!(0, before[1] as usize % 2);

            // It was the second pointer so we flip the bottom bit.
            NonNull::new(before[1].wrapping_byte_add(1))
        } else {
            unreachable!()
        };

        Self { data, meta }
    }

    /// Convert this pointer into a `NonNull<T>`.
    ///
    /// # Safety
    ///
    /// `self` **MUST** have been created by a call to either:
    /// * `AnyPtr::of::<T>(ptr)`
    /// * `AnyPtr::of_mut::<T>(ptr)`
    pub unsafe fn to_ptr<T: ?Sized>(self) -> NonNull<T> {
        const {
            assert!(
                size_of::<*const T>() == size_of::<NonNull<T>>()
                    || size_of::<*const T>() == size_of::<[*mut (); 2]>()
            )
        }
        let mut slot = MaybeUninit::<NonNull<T>>::uninit();
        if let Some(meta) = self.meta {
            assert_eq!(size_of::<*const T>(), size_of::<[*mut (); 2]>(),);
            // We do the reverse convertion from the end of `of_unsized`.
            let ptr = match meta.addr().get() % 2 {
                0 => [meta.as_ptr(), self.data.as_ptr()],
                1 => [self.data.as_ptr(), meta.as_ptr().wrapping_byte_sub(1)],
                _ => unreachable!(),
            };

            // SAFETY: We only have `meta == None` when `T: Sized`
            // and thus the size of `NonNull<[*mut (); 2]>` is the size
            // of `NonNull<T>`.
            let ptr =
                unsafe { transmute_copy::<[*mut (); 2], NonNull<T>>(&ptr) };

            slot.write(ptr);

            // SAFETY: We just initialized the data.
            // We have also reconstructed the pointer data with the same
            // values and in the same order as the original created in
            // `of_unsized::<T>`, thus preserving vtable invariants.
            unsafe { slot.assume_init() }
        } else {
            assert_eq!(size_of::<*const T>(), size_of::<NonNull<T>>());
            assert_eq!(size_of::<NonNull<T>>(), size_of::<NonNull<()>>());
            // SAFETY: We only have `meta == None` when `T: Sized`
            // and thus the size of `NonNull<()>` is the size of `NonNull<T>`.
            let ptr = unsafe {
                transmute_copy::<NonNull<()>, NonNull<T>>(&self.data)
            };

            slot.write(ptr);
            // SAFETY: We just initialized the data.
            unsafe { slot.assume_init() }
        }
    }
}
