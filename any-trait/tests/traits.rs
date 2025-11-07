use any_trait::{AnySubTrait, AnyTrait, AsAnyTrait};

#[test]
fn no_subtraits() {
    #[derive(AnySubTrait, PartialEq, Eq)]
    struct C {
        val: usize,
    }

    let s = C { val: 42 };

    let a = s.as_anytrait();

    match a.downcast_ref::<C>() {
        None => assert!(false, "can't downcast to concrete"),
        Some(s_ref) => {
            assert!(s_ref == &s, "concrete not equal");
        }
    }
}

#[test]
fn multi_traits() {
    trait TA {
        fn add_one(&self) -> usize;
        fn use_x(&self, x: usize) -> usize;
    }
    trait TB: AnyTrait {
        fn add_two(&self) -> usize;
        fn use_x(&self, x: usize) -> usize;
    }
    #[derive(AnySubTrait, PartialEq, Eq)]
    #[any_sub_trait(TA, TB)]
    struct C {
        val: usize,
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

    match a.downcast_ref::<C>() {
        None => assert!(false, "can't downcast to concrete"),
        Some(s_ref) => {
            assert!(s_ref == &s, "concrete not equal");
            assert!(
                s_ref.add_one() == 43,
                "concrete add_one: {}",
                s_ref.add_one()
            );
            assert!(
                s_ref.add_two() == 44,
                "concrete add_two: {}",
                s_ref.add_two()
            );
        }
    }
    match a.downcast_ref::<dyn TA>() {
        None => assert!(false, "can't downcast to TA"),
        Some(ta_ref) => {
            assert!(ta_ref.add_one() == 43, "TA add_one: {}", ta_ref.add_one());
            assert!(ta_ref.use_x(40) == 82, "TA use_x: {}", ta_ref.use_x(40));
        }
    }
    match a.downcast_ref::<dyn TB>() {
        None => assert!(false, "can't downcast to TB"),
        Some(tb_ref) => {
            assert!(tb_ref.add_two() == 44, "TB add_two: {}", tb_ref.add_two());
            assert!(tb_ref.use_x(40) == 2, "TB use_x: {}", tb_ref.use_x(40));

            // You can also cast from TB to AnyTrait and to TA again
            // since TB requires `AnyTrait`
            let a2 = tb_ref.as_anytrait();
            match a2.downcast_ref::<dyn TA>() {
                None => assert!(false, "can't side-cast from TB to TA"),
                Some(ta2_ref) => {
                    assert!(
                        ta2_ref.add_one() == 43,
                        "TA add_one: {}",
                        ta2_ref.add_one()
                    );
                    assert!(
                        ta2_ref.use_x(40) == 82,
                        "TA use_x: {}",
                        ta2_ref.use_x(40)
                    );
                }
            }
        }
    }
}
