#![feature(const_trait_impl)]
use any_trait::typeidconst::TypeIdConst;
//#![feature(generic_const_exprs)]
use ::any_trait::{AnyTrait, AnyTraitCast, AsAnyTrait};

#[test]
fn no_subtraits2() {
    #[derive(PartialEq, Eq)]
    struct C {
        val: usize,
    }
    impl const ::any_trait::typeidconst::TypeIdConstList for C {
        const LEN: usize = 2;
        fn subtraits<const LEN: usize>()
        -> [::any_trait::typeidconst::TypeIdConst; LEN] {
            if LEN != C::LEN {
                panic!("nope. go away.");
            }
            ::any_trait::typeidconst::append_array::<C, 0, LEN>([])
        }
        fn find_in_list(t: &TypeIdConst) -> Option<usize> {
            const LIST: [TypeIdConst; 2] = C::subtraits::<2>();
            let mut i = 0;
            while i < 2 {
                if LIST[i].eq(t) {
                    return Some(i);
                }
                i = i + 1;
            }
            None
        }
    }

    impl AnyTrait for C
    where
        C:,
    {
        fn type_ids(&self) -> &'static [::any_trait::typeidconst::TypeIdConst] {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 2] =
                ::any_trait::typeidconst::append_array::<C, 0, 2>([]);
            &TRAITS
        }
        fn type_erase_mut(
            &mut self,
            trait_num: usize,
        ) -> ::any_trait::anyptr::AnyPtr {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 2] =
                ::any_trait::typeidconst::append_array::<C, 0, 2>([]);
            match trait_num {
                0 => {
                    let ptr = self as *mut dyn AnyTrait;
                    let erased = ::any_trait::anyptr::AnyPtr::from_mut::<
                        dyn AnyTrait,
                    >(ptr);
                    return erased;
                }
                1 => {
                    let ptr = self as *mut C;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from_mut::<C>(ptr);
                    return erased;
                }
                _ => {
                    panic!("AnyTrait: forced cast to wrong type idx");
                }
            }
        }
        fn type_erase(&self, trait_num: usize) -> ::any_trait::anyptr::AnyPtr {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 2] =
                ::any_trait::typeidconst::append_array::<C, 0, 2>([]);
            match trait_num {
                0 => {
                    let ptr = self as *const dyn AnyTrait;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from::<dyn AnyTrait>(ptr);
                    return erased;
                }
                1 => {
                    let ptr = self as *const C;
                    let erased = ::any_trait::anyptr::AnyPtr::from::<C>(ptr);
                    return erased;
                }
                _ => {
                    panic!("AnyTrait: forced cast to wrong type idx");
                }
            }
        }
    }
    let s = C { val: 42 };
    let a = s.as_anytrait();
    match a.cast_ref::<C>() {
        None => {
            if !false {
                {
                    panic!("can\'t cast to concrete");
                }
            }
        }
        Some(s_ref) => {
            if !(s_ref == &s) {
                {
                    panic!("concrete not equal");
                }
            }
        }
    }
}
#[test]
fn multi_traits2() {
    trait TA {
        fn add_one(&self) -> usize;
        fn use_x(&self, x: usize) -> usize;
    }
    trait TB: AnyTrait {
        fn add_two(&self) -> usize;
        fn use_x(&self, x: usize) -> usize;
    }
    #[derive(PartialEq, Eq)]
    struct C {
        val: usize,
    }
    /*
    impl const ::any_trait::typeidconst::TypeIdConstList for C {
        fn subtraits() -> [TypeIdConst] {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 4] =
                ::any_trait::typeidconst::append_array::<C, 2, 4>([
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TA>(),
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TB>(),
                ]);
            TRAITS
        }
    }
    */
    impl AnyTrait for C
    where
        C: TA + TB,
    {
        fn type_ids(&self) -> &'static [::any_trait::typeidconst::TypeIdConst] {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 4] =
                ::any_trait::typeidconst::append_array::<C, 2, 4>([
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TA>(),
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TB>(),
                ]);
            &TRAITS
        }
        fn type_erase_mut(
            &mut self,
            trait_num: usize,
        ) -> ::any_trait::anyptr::AnyPtr {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 4] =
                ::any_trait::typeidconst::append_array::<C, 2, 4>([
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TA>(),
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TB>(),
                ]);
            const N_2: usize =
                ::any_trait::typeidconst::find_in::<dyn TA, 4>(TRAITS);
            const N_3: usize =
                ::any_trait::typeidconst::find_in::<dyn TB, 4>(TRAITS);
            match trait_num {
                0 => {
                    let ptr = self as *mut dyn AnyTrait;
                    let erased = ::any_trait::anyptr::AnyPtr::from_mut::<
                        dyn AnyTrait,
                    >(ptr);
                    return erased;
                }
                1 => {
                    let ptr = self as *mut C;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from_mut::<C>(ptr);
                    return erased;
                }
                N_2 => {
                    let ptr = self as *mut dyn TA;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from_mut::<dyn TA>(ptr);
                    return erased;
                }
                N_3 => {
                    let ptr = self as *mut dyn TB;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from_mut::<dyn TB>(ptr);
                    return erased;
                }
                _ => {
                    panic!("AnyTrait: forced cast to wrong type idx");
                }
            }
        }
        fn type_erase(&self, trait_num: usize) -> ::any_trait::anyptr::AnyPtr {
            const TRAITS: [::any_trait::typeidconst::TypeIdConst; 4] =
                ::any_trait::typeidconst::append_array::<C, 2, 4>([
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TA>(),
                    ::any_trait::typeidconst::TypeIdConst::of::<dyn TB>(),
                ]);
            const N_2: usize =
                ::any_trait::typeidconst::find_in::<dyn TA, 4>(TRAITS);
            const N_3: usize =
                ::any_trait::typeidconst::find_in::<dyn TB, 4>(TRAITS);
            match trait_num {
                0 => {
                    let ptr = self as *const dyn AnyTrait;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from::<dyn AnyTrait>(ptr);
                    return erased;
                }
                1 => {
                    let ptr = self as *const C;
                    let erased = ::any_trait::anyptr::AnyPtr::from::<C>(ptr);
                    return erased;
                }
                N_2 => {
                    let ptr = self as *const dyn TA;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from::<dyn TA>(ptr);
                    return erased;
                }
                N_3 => {
                    let ptr = self as *const dyn TB;
                    let erased =
                        ::any_trait::anyptr::AnyPtr::from::<dyn TB>(ptr);
                    return erased;
                }
                _ => {
                    panic!("AnyTrait: forced cast to wrong type idx");
                }
            }
        }
    }
    impl TA for C {
        fn add_one(&self) -> usize {
            self.val + 1
        }
        fn use_x(&self, x: usize) -> usize {
            self.val + x
        }
    }
    impl TB for C {
        fn add_two(&self) -> usize {
            self.val + 2
        }
        fn use_x(&self, x: usize) -> usize {
            self.val - x
        }
    }
    let s = C { val: 42 };
    let a = s.as_anytrait();
    match a.cast_ref::<C>() {
        None => {
            if !false {
                {
                    panic!("can\'t cast to concrete");
                }
            }
        }
        Some(s_ref) => {
            if !(s_ref == &s) {
                {
                    panic!("concrete not equal");
                }
            }
            if !(s_ref.add_one() == 43) {
                {
                    panic!("concrete add_one: {0}", s_ref.add_one());
                }
            }
            if !(s_ref.add_two() == 44) {
                {
                    panic!("concrete add_two: {0}", s_ref.add_two());
                }
            }
        }
    }
    match a.cast_ref::<dyn TA>() {
        None => {
            if !false {
                {
                    panic!("can\'t cast to TA");
                }
            }
        }
        Some(ta_ref) => {
            if !(ta_ref.add_one() == 43) {
                {
                    panic!("TA add_one: {0}", ta_ref.add_one());
                }
            }
            if !(ta_ref.use_x(40) == 82) {
                {
                    panic!("TA use_x: {0}", ta_ref.use_x(40));
                }
            }
        }
    }
    match a.cast_ref::<dyn TB>() {
        None => {
            if !false {
                {
                    panic!("can\'t cast to TB");
                }
            }
        }
        Some(tb_ref) => {
            if !(tb_ref.add_two() == 44) {
                {
                    panic!("TB add_two: {0}", tb_ref.add_two());
                }
            }
            if !(tb_ref.use_x(40) == 2) {
                {
                    panic!("TB use_x: {0}", tb_ref.use_x(40));
                }
            }
            let x = tb_ref.cast_ref::<dyn TA>();
            if !x.is_some() {
                {
                    panic!("failed cast TB->TA");
                }
            }
            let a2 = tb_ref.as_anytrait();
            match a2.cast_ref::<dyn TA>() {
                None => {
                    if !false {
                        {
                            panic!("can\'t side-cast from TB to TA");
                        }
                    }
                }
                Some(ta2_ref) => {
                    if !(ta2_ref.add_one() == 43) {
                        {
                            panic!("TA add_one: {0}", ta2_ref.add_one());
                        }
                    }
                    if !(ta2_ref.use_x(40) == 82) {
                        {
                            panic!("TA use_x: {0}", ta2_ref.use_x(40));
                        }
                    }
                }
            }
        }
    }
}
