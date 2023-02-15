use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DataEnum, DataStruct, Ident, Path, Visibility};

pub fn derive_arbitrary_struct(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    data: &DataStruct,
) -> TokenStream2 {
    derive_arbitrary_struct_common(path, vis, ident, data, FieldType::Named)
}

pub fn derive_arbitrary_struct_tuple(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    data: &DataStruct,
) -> TokenStream2 {
    derive_arbitrary_struct_common(path, vis, ident, data, FieldType::Unnamed)
}

enum FieldType {
    Named,
    Unnamed,
}

fn derive_arbitrary_struct_common(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    data: &DataStruct,
    field_type: FieldType,
) -> TokenStream2 {
    let arbitrary_type_ident = format_ident!("Arbitrary{}", ident);
    let mod_ident = format_ident!("mod_{}", ident);

    let arbitrary_type_fields: Vec<TokenStream2> = data
        .fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            match &field.ident {
                Some(ident) => {
                    quote! {
                        #ident: <#field_type as #path::arbitrary::SorobanArbitrary>::Prototype
                    }
                }
                None => {
                    quote! {
                        <#field_type as #path::arbitrary::SorobanArbitrary>::Prototype
                    }
                }
            }
        })
        .collect();

    let field_conversions: Vec<TokenStream2> = data
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| match &field.ident {
            Some(ident) => {
                quote! {
                    #ident: v.#ident.into_val(env)
                }
            }
            None => {
                let i = syn::Index::from(i);
                quote! {
                    v.#i.into_val(env)
                }
            }
        })
        .collect();

    let arbitrary_type_decl = match field_type {
        FieldType::Named => quote! {
            struct #arbitrary_type_ident {
                #(#arbitrary_type_fields,)*
            }
        },
        FieldType::Unnamed => quote! {
            struct #arbitrary_type_ident (
                #(#arbitrary_type_fields,)*
            );
        },
    };

    let arbitrary_ctor = match field_type {
        FieldType::Named => quote! {
            #ident {
                #(#field_conversions,)*
            }
        },
        FieldType::Unnamed => quote! {
            #ident (
                #(#field_conversions,)*
            )
        },
    };

    quote_arbitrary(
        path,
        vis,
        ident,
        mod_ident,
        arbitrary_type_ident,
        arbitrary_type_decl,
        arbitrary_ctor,
    )
}

pub fn derive_arbitrary_enum(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    data: &DataEnum,
) -> TokenStream2 {
    let arbitrary_type_ident = format_ident!("Arbitrary{}", ident);
    let mod_ident = format_ident!("mod_{}", ident);

    let arbitrary_type_variants: Vec<TokenStream2> = data
        .variants
        .iter()
        .map(|variant| {
            let mut field_types = None;
            let variant_ident = &variant.ident;
            let fields: Vec<TokenStream2> = variant
                .fields
                .iter()
                .map(|field| {
                    let field_type = &field.ty;
                    match &field.ident {
                        Some(ident) => {
                            field_types = Some(FieldType::Named);
                            quote! {
                                #ident: <#field_type as #path::arbitrary::SorobanArbitrary>::Prototype
                            }
                        }
                        None => {
                            field_types = Some(FieldType::Unnamed);
                            quote! {
                                <#field_type as #path::arbitrary::SorobanArbitrary>::Prototype
                            }
                        }
                    }
                })
                .collect();
            match field_types {
                None => {
                    quote! {
                        #variant_ident
                    }
                },
                Some(FieldType::Named) => {
                    quote! {
                        #variant_ident { #(#fields,)* }
                    }
                }
                Some(FieldType::Unnamed) => {
                    quote! {
                        #variant_ident ( #(#fields,)* )
                    }
                }
            }
        })
        .collect();

    let variant_conversions: Vec<TokenStream2> = data
        .variants
        .iter()
        .map(|variant| {
            let mut field_types = None;
            let variant_ident = &variant.ident;
            let fields: Vec<TokenStream2> = variant
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    match &field.ident {
                        Some(ident) => {
                            quote! {
                                #ident
                            }
                        }
                        None => {
                            let ident = format_ident!("field_{}", i);
                            quote! {
                                #ident
                            }
                        }
                    }
                })
                .collect();
            let field_conversions: Vec<TokenStream2> = variant
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    match &field.ident {
                       Some(ident) => {
                            field_types = Some(FieldType::Named);
                            quote! {
                                #ident: #ident.into_val(env)
                            }
                        }
                        None => {
                            field_types = Some(FieldType::Unnamed);
                            let ident = format_ident!("field_{}", i);
                            quote! {
                                #ident.into_val(env)
                            }
                        }
                    }
                })
                .collect();
            match field_types {
                None => {
                    quote! {
                        #arbitrary_type_ident::#variant_ident => #ident::#variant_ident
                    }
                },
                Some(FieldType::Named) => {
                    quote! {
                        #arbitrary_type_ident::#variant_ident { #(#fields,)* } => #ident::#variant_ident { #(#field_conversions,)* }
                    }
                }
                Some(FieldType::Unnamed) => {
                    quote! {
                        #arbitrary_type_ident::#variant_ident ( #(#fields,)* ) => #ident::#variant_ident ( #(#field_conversions,)* )
                    }
                }
            }
        })
        .collect();

    let arbitrary_type_decl = quote! {
        enum #arbitrary_type_ident {
            #(#arbitrary_type_variants,)*
        }
    };
    let arbitrary_ctor = quote! {
        match v {
            #(#variant_conversions,)*
        }
    };

    quote_arbitrary(
        path,
        vis,
        ident,
        mod_ident,
        arbitrary_type_ident,
        arbitrary_type_decl,
        arbitrary_ctor,
    )
}

pub fn derive_arbitrary_enum_int(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    data: &DataEnum,
) -> TokenStream2 {
    let arbitrary_type_ident = format_ident!("Arbitrary{}", ident);
    let mod_ident = format_ident!("mod_{}", ident);

    let arbitrary_type_variants: Vec<TokenStream2> = data
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            quote! {
                #variant_ident
            }
        })
        .collect();

    let variant_conversions: Vec<TokenStream2> = data
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            quote! {
                #arbitrary_type_ident::#variant_ident => #ident::#variant_ident
            }
        })
        .collect();

    let arbitrary_type_decl = quote! {
        enum #arbitrary_type_ident {
            #(#arbitrary_type_variants,)*
        }
    };
    let arbitrary_ctor = quote! {
        match v {
            #(#variant_conversions,)*
        }
    };

    quote_arbitrary(
        path,
        vis,
        ident,
        mod_ident,
        arbitrary_type_ident,
        arbitrary_type_decl,
        arbitrary_ctor,
    )
}

fn quote_arbitrary(
    path: &Path,
    vis: &Visibility,
    ident: &Ident,
    mod_ident: Ident,
    arbitrary_type_ident: Ident,
    arbitrary_type_decl: TokenStream2,
    arbitrary_ctor: TokenStream2,
) -> TokenStream2 {
    let arb_vis = get_arb_vis(vis);

    // This derive is complicated by some constraints:
    //
    // #[derive(Arbitrary)] expects `std` and `arbitrary` to be in scope.
    //
    // To reduce risks of collisions with `arbitrary` we
    // put everything into a submodule.
    //
    // This makes getting visibility modifiers correct
    // for all cases challenging.

    quote! {
        #[cfg(feature = "arbitrary")]
        #[allow(non_snake_case)]
        mod #mod_ident {
            use super::*;

            use #path::arbitrary::std;
            use #path::arbitrary::arbitrary;
            // fixme fully-qualify all uses of this trait
            use #path::IntoVal;

            #[cfg(feature = "arbitrary")]
            #[derive(#path::arbitrary::arbitrary::Arbitrary, Debug)]
            #arb_vis #arbitrary_type_decl

            #[cfg(feature = "arbitrary")]
            impl #path::arbitrary::SorobanArbitrary for #ident {
                type Prototype = #arbitrary_type_ident;
            }

            #[cfg(feature = "arbitrary")]
            impl #path::arbitrary::SorobanArbitraryPrototype for #arbitrary_type_ident {
                type Into = #ident;
            }

            #[cfg(feature = "arbitrary")]
            impl #path::TryFromVal<#path::Env, #arbitrary_type_ident> for #ident {
                type Error = #path::ConversionError;
                fn try_from_val(env: &#path::Env, v: &#arbitrary_type_ident) -> std::result::Result<Self, Self::Error> {
                    Ok(#arbitrary_ctor)
                }
            }
        }
    }
}

fn get_arb_vis(vis: &Visibility) -> TokenStream2 {
    // todo explain this mess
    match vis {
        Visibility::Public(_) => quote! { pub },
        Visibility::Inherited => quote! { pub(super) },
        Visibility::Crate(_) => syn::Error::new_spanned(vis, "unsupported crate visibility")
            .to_compile_error()
            .into(),
        Visibility::Restricted(vis) => match (vis.in_token, vis.path.get_ident()) {
            (None, Some(ident)) => {
                let crate_ident = &Ident::new("crate", ident.span());
                if ident == crate_ident {
                    quote! { pub(crate) }
                } else {
                    syn::Error::new_spanned(vis, "unsupported restricted visibility")
                        .to_compile_error()
                        .into()
                }
            }
            _ => syn::Error::new_spanned(vis, "unsupported restricted visibility")
                .to_compile_error()
                .into(),
        },
    }
}
