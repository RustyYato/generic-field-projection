extern crate proc_macro;

use proc_macro_roids::*;

use proc_macro::TokenStream;
use quote::quote;

use proc_macro2;
use syn;

#[proc_macro_derive(Field)]
pub fn derive_field(ty: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(ty as syn::DeriveInput);

    match ty.data {
        syn::Data::Struct(_) => derive_struct(ty),
        syn::Data::Union(_) => derive_union(ty),
        syn::Data::Enum(_) => panic!(),
    }
}

macro_rules! item {
    ($($tokens:tt)*) => {{
        let quote = TokenStream::from(quote!($($tokens)*));
        syn::parse_macro_input!(quote as syn::Item)
    }}
}

macro_rules! expr {
    ($($tokens:tt)*) => {{
        let quote = TokenStream::from(quote!($($tokens)*));
        syn::parse_macro_input!(quote as syn::Expr)
    }}
}

// macro_rules! attr {
//     ($($tokens:tt)*) => {{
//         let quote = TokenStream::from(quote!($($tokens)*));
//         syn::parse_macro_input!(quote as syn::Attribute)
//     }}
// }

fn derive_struct(ty: syn::DeriveInput) -> TokenStream {
    match ty.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(_),
            ..
        }) => derive_named(ty),
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(_),
            ..
        }) => derive_unnamed(ty),
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => syn::Error::new(ty.ident.span(), "Unit structs are not supported")
            .to_compile_error()
            .into(),
        _ => unreachable!(),
    }
}

fn derive_named(ty: syn::DeriveInput) -> TokenStream {
    let syn::DeriveInput {
        vis,
        ident: input_ident,
        generics,
        data,
        ..
    } = ty;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = data
    {
        fields
    } else {
        unreachable!()
    };

    let module_name = input_ident.append("_fields");

    let mut module = new_module(module_name.clone());
    module.vis = vis;

    let contents = &mut module.content.as_mut().unwrap().1;

    let mut fields_marker = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

    contents.push(item!(
        use super::*;
    ));

    let (generic_header, generic, where_clause) = generics.split_for_impl();
    for field in fields.named {
        let ident = field.ident.unwrap();

        contents.push(item!(
            #[allow(non_camel_case_types)]
            pub struct #ident<T>(::gfp_core::derive::Invariant<T>);
        ));

        contents.push(item!(
            impl<T> #ident<T> {
                pub const INIT: Self = Self(::gfp_core::derive::Invariant(::gfp_core::derive::PhantomData));
            }
        ));

        contents.push(item!(
            impl<T> Clone for #ident<T> {
                fn clone(&self) -> Self { *self }
            }
        ));

        contents.push(item!(
            impl<T> Copy for #ident<T> {}
        ));

        let ty = &field.ty;

        contents.push(item!(
            unsafe impl #generic_header ::gfp_core::Field for #ident<super::#input_ident #generic> {
                type Parent = super::#input_ident #generic;
                type Type = #ty;
                type Name = ::gfp_core::derive::Once<&'static str>;

                #[inline]
                fn name(&self) -> Self::Name {
                    ::gfp_core::derive::once(stringify!(#ident))
                }

                #[inline]
                unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
                    ::gfp_core::ptr_project!(const ptr #ident)
                }

                #[inline]
                unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
                    ::gfp_core::ptr_project!(mut ptr #ident)
                }
            }
        ));

        let ty = TokenStream::from(quote!(
            #module_name::#ident<#input_ident #generic>
        ));
        let ty = syn::parse_macro_input!(ty as syn::Type);

        fields_new.push(expr!(
            #ident: #module_name::#ident::INIT
        ));

        let item = syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token![pub](proc_macro2::Span::call_site()),
            }),
            ident: Some(ident),
            colon_token: field.colon_token,
            ty,
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {

        #[allow(non_snake_case)]
        #module

        struct #field_type_name #generic_header #where_clause {
            #fields_marker
        }

        impl#generic_header #input_ident #generic #where_clause {
            const FIELDS: #field_type_name #generic = #field_type_name {
                #fields_new
            };

            fn fields() -> #field_type_name #generic {
                #field_type_name {
                    #fields_new
                }
            }
        }
    })
}

fn derive_unnamed(ty: syn::DeriveInput) -> TokenStream {
    let syn::DeriveInput {
        vis,
        ident: input_ident,
        generics,
        data,
        ..
    } = ty;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Unnamed(fields),
        ..
    }) = data
    {
        fields
    } else {
        unreachable!()
    };

    let module_name = input_ident.append("_fields");

    let mut module = new_module(module_name.clone());
    module.vis = vis;

    let contents = &mut module.content.as_mut().unwrap().1;

    let mut fields_marker = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

    contents.push(item!(
        use super::*;
    ));

    let (generic_header, generic, where_clause) = generics.split_for_impl();
    for (i, field) in fields.unnamed.iter().enumerate() {
        let ident = syn::Ident::new(&format!("_{}", i), proc_macro2::Span::call_site());

        contents.push(item!(
            #[allow(non_camel_case_types)]
            pub struct #ident<T>(::gfp_core::derive::Invariant<T>);
        ));

        contents.push(item!(
            impl<T> #ident<T> {
                pub const INIT: Self = Self(::gfp_core::derive::Invariant(::gfp_core::derive::PhantomData));
            }
        ));

        contents.push(item!(
            impl<T> Clone for #ident<T> {
                fn clone(&self) -> Self { *self }
            }
        ));

        contents.push(item!(
            impl<T> Copy for #ident<T> {}
        ));

        let ty = &field.ty;

        let index = syn::Member::Unnamed(syn::Index {
            index: i as u32,
            span: proc_macro2::Span::call_site(),
        });

        contents.push(item!(
            unsafe impl #generic_header ::gfp_core::Field for #ident<super::#input_ident #generic> {
                type Parent = super::#input_ident #generic;
                type Type = #ty;
                type Name = ::gfp_core::derive::Once<&'static str>;

                #[inline]
                fn name(&self) -> Self::Name {
                    ::gfp_core::derive::once(stringify!(#index))
                }

                #[inline]
                unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
                    &(*ptr).#index
                }

                #[inline]
                unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
                    &mut (*ptr).#index
                }
            }
        ));

        let ty = TokenStream::from(quote!(
            #module_name::#ident<#input_ident #generic>
        ));
        let ty = syn::parse_macro_input!(ty as syn::Type);

        fields_new.push(expr!(
            #module_name::#ident::INIT
        ));

        let item = syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token![pub](proc_macro2::Span::call_site()),
            }),
            ident: field.ident.clone(),
            colon_token: field.colon_token,
            ty,
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {

        #[allow(non_snake_case)]
        #module

        struct #field_type_name #generic_header(#fields_marker) #where_clause;

        impl#generic_header #input_ident #generic #where_clause {
            const FIELDS: #field_type_name #generic = #field_type_name(#fields_new);

            fn fields() -> #field_type_name #generic {
                #field_type_name(#fields_new)
            }
        }
    })
}

fn derive_union(ty: syn::DeriveInput) -> TokenStream {
    let syn::DeriveInput {
        vis,
        ident: input_ident,
        generics,
        data,
        ..
    } = ty;

    let fields = if let syn::Data::Union(syn::DataUnion { fields, .. }) = data {
        fields
    } else {
        unreachable!()
    };

    let module_name = input_ident.append("_fields");

    let mut module = new_module(module_name.clone());
    module.vis = vis;

    let contents = &mut module.content.as_mut().unwrap().1;

    let mut fields_marker = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

    contents.push(item!(
        use super::*;
    ));

    let (generic_header, generic, where_clause) = generics.split_for_impl();
    for field in fields.named {
        let ident = field.ident.unwrap();

        contents.push(item!(
            #[allow(non_camel_case_types)]
            pub struct #ident<T>(::gfp_core::derive::Invariant<T>);
        ));

        contents.push(item!(
            impl<T> #ident<T> {
                pub const unsafe fn init() -> Self {
                    Self(::gfp_core::derive::Invariant(::gfp_core::derive::PhantomData))
                }
            }
        ));

        contents.push(item!(
            impl<T> Clone for #ident<T> {
                fn clone(&self) -> Self { *self }
            }
        ));

        contents.push(item!(
            impl<T> Copy for #ident<T> {}
        ));

        let ty = &field.ty;

        contents.push(item!(
            unsafe impl #generic_header ::gfp_core::Field for #ident<super::#input_ident #generic> {
                type Parent = super::#input_ident #generic;
                type Type = #ty;
                type Name = ::gfp_core::derive::Once<&'static str>;

                #[inline]
                fn name(&self) -> Self::Name {
                    ::gfp_core::derive::once(stringify!(#input_ident))
                }

                #[inline]
                unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
                    &(*ptr).#ident
                }

                #[inline]
                unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
                    &mut (*ptr).#ident
                }
            }
        ));

        let ty = TokenStream::from(quote!(
            #module_name::#ident<#input_ident #generic>
        ));
        let ty = syn::parse_macro_input!(ty as syn::Type);

        fields_new.push(expr!(
            #ident: #module_name::#ident::init()
        ));

        let item = syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token![pub](proc_macro2::Span::call_site()),
            }),
            ident: Some(ident),
            colon_token: field.colon_token,
            ty,
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {

        #[allow(non_snake_case)]
        #module

        struct #field_type_name #generic_header #where_clause {
            #fields_marker
        }

        impl#generic_header #input_ident #generic #where_clause {
            unsafe fn fields() -> #field_type_name #generic {
                #field_type_name {
                    #fields_new
                }
            }
        }
    })
}

fn new_module(ident: syn::Ident) -> syn::ItemMod {
    syn::ItemMod {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mod_token: syn::Token![mod](proc_macro2::Span::call_site()),
        ident,
        content: Some((
            syn::token::Brace {
                span: proc_macro2::Span::call_site(),
            },
            Vec::new(),
        )),
        semi: None,
    }
}
