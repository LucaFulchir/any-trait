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
        impl AnyTrait for #name
          where #name: #(#extra_traits)+*
        {
            fn type_ids(&self) -> &'static [::any_trait::typeidconst::TypeIdConst] {
                static TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*]));
                &TRAITS
            }
            unsafe fn cast_to_mut(&mut self, trait_num: usize) -> usize {
                const TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*]));
                // Only the second part if TRAITS is ordered.
                // but  that means that the macro does not know the index of
                // a type in that list.
                // create constants that we can `match` against.
                #(const #trait_idx_name :usize =
                    ::any_trait::typeidconst::find_in::
                        <dyn #extra_traits, #tot_traits>(TRAITS);
                )*
                #[allow(unsafe_code)]
                unsafe {
                    // Here the horror happens.
                    // we cast `self` to the correct dyn type.
                    // but `*const dyn ...` is a fat pointer,
                    // which is not guaranteed to have a stable size between
                    // rust versions. so we use `*const *const` and have
                    // a clean pointer,
                    // that we horrifiyingly reinterpret as usize
                    // ...so much for typesafety!
                    match trait_num {
                        0 => {
                            union U {
                                ptr: *mut *mut dyn AnyTrait,
                                raw_ptr: usize,
                            }
                            let t = &mut *(self as *mut dyn AnyTrait);
                            let tmp = U {
                                ptr: &mut (t as *mut dyn AnyTrait),
                            };
                            return tmp.raw_ptr;
                        },
                        1 => {
                            union U {
                                ptr: *mut *mut #name,
                                raw_ptr: usize,
                            }
                            let mut p = self as *mut #name;
                            let tmp = U { ptr: &mut p, };

                            return tmp.raw_ptr;
                        }
                        #(#trait_idx_name => {
                            union U {
                                ptr: *mut *mut dyn #extra_traits,
                                raw_ptr: usize,
                            }
                            let t = &mut *(self as *mut dyn #extra_traits);
                            let tmp = U {
                                ptr: &mut (t as *mut dyn #extra_traits),
                            };
                            return tmp.raw_ptr;
                        }
                        )*
                        _ => {
                            panic!("AnyTrait: forced cast to wrong type idx")
                        }
                    }
                }
            }
            unsafe fn cast_to(&self, trait_num: usize) -> usize {
                const TRAITS : [::any_trait::typeidconst::TypeIdConst; #tot_traits] =
                    ::any_trait::typeidconst::append_array::
                        <#name, #extra_traits_num, #tot_traits>(
                    ::any_trait::typeidconst::sort_array(
                        [#(::any_trait::typeidconst::TypeIdConst::of::
                            <dyn #extra_traits>()),*]));
                // Only the second part if TRAITS is ordered.
                // but  that means that the macro does not know the index of
                // a type in that list.
                // create constants that we can `match` against.
                #(const #trait_idx_name :usize =
                    ::any_trait::typeidconst::find_in::
                        <dyn #extra_traits, #tot_traits>(TRAITS);
                )*
                #[allow(unsafe_code)]
                unsafe {
                    // Here the horror happens.
                    // we cast `self` to the correct dyn type.
                    // but `*const dyn ...` is a fat pointer,
                    // which is not guaranteed to have a stable size between
                    // rust versions. so we use `*const *const` and have
                    // a clean pointer,
                    // that we horrifiyingly reinterpret as usize
                    // ...so much for typesafety!
                    match trait_num {
                        0 => {
                            union U {
                                ptr: *const *const dyn AnyTrait,
                                raw_ptr: usize,
                            }
                            let t2 = &*(self as *const dyn AnyTrait);
                            let tmp = U {
                                ptr: &(t2 as *const dyn AnyTrait),
                            };
                            return tmp.raw_ptr;
                        },
                        1 => {
                            union U {
                                ptr: *const *const #name,
                                raw_ptr: usize,
                            }
                            let p = self as *const #name;
                            let tmp = U { ptr: &p };

                            return tmp.raw_ptr;
                        }
                        #(#trait_idx_name => {
                            union U {
                                ptr: *const *const dyn #extra_traits,
                                raw_ptr: usize,
                            }
                            let t2 = &*(self as *const dyn #extra_traits);
                            let tmp = U {
                                ptr: &(t2 as *const dyn #extra_traits),
                            };
                            return tmp.raw_ptr;
                        }
                        )*
                        _ => {
                            panic!("AnyTrait: forced cast to wrong type idx")
                        }
                    }
                }
            }
        }
    };
    TokenStream::from(out)
}
