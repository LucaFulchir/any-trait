//! Internal implementation of a const-comparable TypeId
//!
//! You should not be here. go away.

/// TypeId, but const-comparable, sortable
///
/// uses `::core::any::type_name::<T>()` underneath
#[derive(Copy, Clone)]
pub struct TypeIdConst {
    // I wanted to use ::core::any::TypeId
    // turns out I can't because it does not implement Ord
    // and I can't put it into an union and compare raw bytes, either.
    s: &'static str,
}

impl TypeIdConst {
    pub const fn of<T: ?Sized + 'static>() -> TypeIdConst {
        return TypeIdConst {
            s: ::core::any::type_name::<T>(),
        };
    }
    pub const fn cmp(&self, other: Self) -> ::core::cmp::Ordering {
        let self_bytes = self.s.as_bytes();
        let other_bytes = other.s.as_bytes();
        let mut i: usize = 0;
        while i < self_bytes.len() {
            if other_bytes.len() < i {
                return ::core::cmp::Ordering::Greater;
            }
            if self_bytes[i] < other_bytes[i] {
                return ::core::cmp::Ordering::Less;
            }
            if self_bytes[i] > other_bytes[i] {
                return ::core::cmp::Ordering::Greater;
            }
            i = i + 1;
        }
        if self_bytes.len() > i {
            return ::core::cmp::Ordering::Less;
        }
        return ::core::cmp::Ordering::Equal;
    }
}
impl ::core::cmp::PartialEq for TypeIdConst {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(*other).is_eq()
    }
}
impl ::core::cmp::Eq for TypeIdConst {}

impl ::core::cmp::PartialOrd for TypeIdConst {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(*other))
    }
}
impl ::core::cmp::Ord for TypeIdConst {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.cmp(*other)
    }
}

/// const sort with insertionsort.
pub const fn sort_array<const N: usize>(
    array: [TypeIdConst; N],
) -> [TypeIdConst; N] {
    // come and tell me this is not performant
    // and I'll implement random_sort
    // joking, but I'm not wasting time on this
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut out: [TypeIdConst; N] = array;
    while i < N {
        let mut min: usize = i;
        while j < N {
            if array[j].cmp(array[i]).is_lt() {
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
        if array[i].cmp(TypeIdConst::of::<T>()).is_eq() {
            return i;
        }
        i = i + 1;
    }
    panic!("TypeIDConst find_in: called with non-member");
}
