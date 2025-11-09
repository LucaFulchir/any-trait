#![no_std]
extern crate alloc;
use ::alloc::vec::Vec;
use ::proc_macro::TokenStream;
use ::quote::quote;
use ::syn::{
    DeriveInput,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// example: `#[any_sub_trait(T1, T2, ...)]`
struct SubTraits(Vec<::syn::Ident>);
impl Parse for SubTraits {
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        use ::syn::Token;
        let mut trait_list = Vec::with_capacity(4);

        let fields = input.parse_terminated(::syn::Ident::parse, Token![,])?;
        fields.into_iter().for_each(|f| {
            trait_list.push(f);
        });
        Ok(SubTraits(trait_list))
    }
}

/// Add the `AnyTrait` implementation
///
/// Usage:
/// ```ignore
/// #[derive(AnySubTrait)]
/// #[any_sub_trait(TraitA, TraitB, ...)] // optional
/// struct MyStruct {}
/// ```
#[proc_macro_derive(AnySubTrait, attributes(any_sub_trait))]
pub fn derive_anytrait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // make sure we are only called on structs
    let ::syn::Data::Struct(_items) = &input.data else {
        use ::syn::spanned::Spanned;
        return ::syn::Error::new(
            //
            input.span(),
            "#[derive(AnyTrait)]: not called on a struct",
        )
        .to_compile_error()
        .into();
    };

    let name = input.ident.clone();
    let extra_traits = input
        .attrs
        .iter()
        .filter_map(|a| {
            if !a.path().is_ident("any_sub_trait") {
                return None;
            }
            match &a.meta {
                ::syn::Meta::List(list) => {
                    let parsed: SubTraits = list.parse_args().unwrap();
                    Some(parsed)
                }
                _ => None,
            }
        })
        .nth(0)
        .or_else(|| Some(SubTraits(Vec::new())))
        .unwrap()
        .0;

    let extra_traits_num: ::syn::Index = ::syn::Index::from(extra_traits.len());
    let tot_traits: ::syn::Index = ::syn::Index::from(2 + extra_traits.len());

    let mut trait_idx_name =
        Vec::<::syn::Ident>::with_capacity(extra_traits.len());
    extra_traits.iter().enumerate().for_each(|(idx, _a)| {
        trait_idx_name.push(::quote::format_ident!("N_{}", idx + 2));
    });

    let out = quote! {
        impl const ::any_trait::typeidconst::TypeIdConstList for #name {
            const LEN: usize = #tot_traits;
            fn subtraits<const LEN: usize>() -> [::any_trait::typeidconst::TypeIdConst;LEN] {
                if LEN != #tot_traits {
                    panic!("nope. go away.");
                }
                ::any_trait::typeidconst::append_array::
                    <#name, #extra_traits_num, LEN>(
                /* waiting for const Ord on TypeId...
                ::any_trait::typeidconst::sort_array(
                    [#(::any_trait::typeidconst::TypeIdConst::of::
                        <dyn #extra_traits>()),*])
                */
                [#(::any_trait::typeidconst::TypeIdConst::of::
                    <dyn #extra_traits>()),*]
                    )
            }
            fn find_in_list(t: &::any_trait::typeidconst::TypeIdConst) -> Option<usize> {
                const LIST: [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    #name::subtraits::<#tot_traits>();
                let mut i = 0;
                while i < #tot_traits {
                    if LIST[i].eq(t) {
                        return Some(i);
                    }
                    i = i + 1;
                }
                None
            }
        }

        impl AnyTrait for #name
          where #name: #(#extra_traits)+*
        {
            fn type_ids(&self) -> &'static [::any_trait::typeidconst::TypeIdConst] {
                const TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    /* waiting for const Ord on TypeId...
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*])
                    */
                    [#(::any_trait::typeidconst::TypeIdConst::of::
                        <dyn #extra_traits>()),*]
                        );
                &TRAITS
            }
            fn type_erase_mut(&mut self, trait_num: usize) -> ::any_trait::anyptr::AnyPtr {
                const TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    /* waiting for const Ord on TypeId
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*])
                    */
                    [#(::any_trait::typeidconst::TypeIdConst::of::
                        <dyn #extra_traits>()),*]
                    );
                // In the future only the second part of TRAITS will be ordered.
                // but  that means that the macro does not know the index of
                // a type in that list.
                // create constants that we can `match` against.
                #(const #trait_idx_name :usize =
                    ::any_trait::typeidconst::find_in::
                        <dyn #extra_traits, #tot_traits>(TRAITS);
                )*
                // Type-erase `self` into `AnyPtr`
                match trait_num {
                        0 => {
                            let ptr = self as *mut dyn AnyTrait;

                        let erased = ::any_trait::anyptr::AnyPtr::from_mut::<dyn AnyTrait>(ptr);
                        return erased;
                    },
                    1 => {
                        let ptr = self as *mut #name;

                        let erased = ::any_trait::anyptr::AnyPtr::from_mut::<#name>(ptr);
                        return erased;
                    }
                    #(#trait_idx_name => {
                        let ptr = self as *mut dyn #extra_traits;

                        let erased = ::any_trait::anyptr::AnyPtr::from_mut::<dyn #extra_traits>(ptr);
                        return erased;
                    }
                    )*
                    _ => {
                        panic!("AnyTrait: forced cast to wrong type idx")
                    }
                }
            }
            fn type_erase(&self, trait_num: usize) -> ::any_trait::anyptr::AnyPtr {
                const TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    /* waiting for const Ord on TypeId
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*])
                    */
                    [#(::any_trait::typeidconst::TypeIdConst::of::
                        <dyn #extra_traits>()),*]
                    );
                // In the future only the second part of TRAITS will be ordered.
                // but  that means that the macro does not know the index of
                // a type in that list.
                // create constants that we can `match` against.
                #(const #trait_idx_name :usize =
                    ::any_trait::typeidconst::find_in::
                        <dyn #extra_traits, #tot_traits>(TRAITS);
                )*
                // Type-erase `self` into `AnyPtr`
                match trait_num {
                    0 => {
                        let ptr = self as *const dyn AnyTrait;

                        let erased = ::any_trait::anyptr::AnyPtr::from::<dyn AnyTrait>(ptr);
                        return erased;
                    },
                    1 => {
                        let ptr = self as *const #name;

                        let erased = ::any_trait::anyptr::AnyPtr::from::<#name>(ptr);
                        return erased;
                    }
                    #(#trait_idx_name => {
                        let ptr = self as *const dyn #extra_traits;

                        let erased = ::any_trait::anyptr::AnyPtr::from::<dyn #extra_traits>(ptr);
                        return erased;
                    }
                    )*
                    _ => {
                        panic!("AnyTrait: forced cast to wrong type idx")
                    }
                }
            }
        }
    };
    TokenStream::from(out)
}
