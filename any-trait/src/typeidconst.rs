//! Internal implementation of a const-comparable TypeId
//!
//! We are missing a const Ord implementation,
//! and we don't have a sound way of doing it.
//! Order is only used for performance,
//! but it's might make a difference when you have > 100 subtraits
//!
//! You should not be here. go away.

/// TypeId, but const-comparable
/// TODO: Make sortable somehow? soundness problems on order. plz help
#[derive(Copy, Clone)]
pub struct TypeIdConst {
    t: ::core::any::TypeId,
}

impl TypeIdConst {
    pub const fn of<T: ?Sized + 'static>() -> TypeIdConst {
        return TypeIdConst {
            t: ::core::any::TypeId::of::<T>(),
        };
    }
    pub const fn eq(&self, other: &Self) -> bool {
        return self.t.eq(&other.t);
    }
    /* Waiting for const Ord on TypeId, or other trick...
    pub const fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        if self.t.eq(&other.t) {
            return ::core::cmp::Ordering::Equal;
        }
        const TYPEID_SIZE: usize =
            ::core::mem::size_of::<::core::any::TypeId>();
        // This is unsound.
        //
        // any trick to do Ord on TypeID?
        let self_bytes: [u8; TYPEID_SIZE];
        let other_bytes: [u8; TYPEID_SIZE];
        unsafe {
            self_bytes = ::core::mem::transmute_copy::<
                ::core::any::TypeId,
                [u8; TYPEID_SIZE],
            >(&self.t);
            other_bytes = ::core::mem::transmute_copy::<
                ::core::any::TypeId,
                [u8; TYPEID_SIZE],
            >(&other.t);
        };
        let mut i = 0;
        while i < TYPEID_SIZE {
            if self_bytes[i] < other_bytes[i] {
                return ::core::cmp::Ordering::Less;
            }
            if self_bytes[i] > other_bytes[i] {
                return ::core::cmp::Ordering::Greater;
            }
            i = i + 1;
        }
        return ::core::cmp::Ordering::Equal;
    }
    */
}
impl ::core::cmp::PartialEq for TypeIdConst {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}
impl ::core::cmp::Eq for TypeIdConst {}

/* Waiting for const Cmp on TypeID
impl ::core::cmp::PartialOrd for TypeIdConst {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl ::core::cmp::Ord for TypeIdConst {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.cmp(other)
    }
}
/// const sort with insertionsort.
pub const fn sort_array<const N: usize>(
    array: [TypeIdConst; N],
) -> [TypeIdConst; N] {
    return array;
    // come and tell me this is not performant
    // and I'll implement random_sort
    // joking, but I'm not wasting time on this
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut out: [TypeIdConst; N] = array;
    while i < N {
        let mut min: usize = i;
        while j < N {
            if array[j].cmp(&array[i]).is_lt() {
                min = j;
            }
            j = j + 1
        }
        if min != i {
            let old = out[i];
            out[i] = out[j];
            out[j] = old;
        }
        i = i + 1;
    }
    out
}
*/

/// get `[TypeIdConst;N]` in input and return `[TypeIdConst;N + 2]`
///
/// we add the first two elements, and they always are:
/// * `TypeIdConst::of::<dyn AnyTrait>`
/// * `TypeIdConst::of::<T>`
pub const fn append_array<T: 'static, const N: usize, const M: usize>(
    array: [TypeIdConst; N],
) -> [TypeIdConst; M] {
    assert!(N + 2 == M, "M needs to be N + 2");
    let mut out: [TypeIdConst; M] = [TypeIdConst::of::<T>(); M];
    out[0] = TypeIdConst::of::<dyn super::AnyTrait>();
    let mut i: usize = 2;
    while i < M {
        out[i] = array[i - 2];
        i = i + 1;
    }

    out
}

/// const-find the `TypeIdConst` of `T` inside the given array.
/// return its index or panic
pub const fn find_in<T: ?Sized + 'static, const N: usize>(
    array: [TypeIdConst; N],
) -> usize {
    let mut i: usize = 0;
    while i < N {
        if array[i].eq(&TypeIdConst::of::<T>()) {
            return i;
        }
        i = i + 1;
    }
    panic!("TypeIDConst find_in: called with non-member");
}
