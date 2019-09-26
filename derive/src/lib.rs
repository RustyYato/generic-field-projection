
extern crate proc_macro;

use proc_macro_roids::*;

use proc_macro::TokenStream;
use quote::quote;

use proc_macro2;
use syn;

#[proc_macro_derive(Field, attributes(pin, default))]
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

fn derive_struct(ty: syn::DeriveInput) -> TokenStream {
    let syn::DeriveInput {
        attrs: _, vis, ident: input_ident, generics, data
    } = ty;

    let data = if let syn::Data::Struct(data) = data {
        data
    } else {
        unreachable!()
    };

    let module_name = input_ident.append("_fields");

    let mut module = new_module(module_name.clone());
    module.vis = vis;

    let contents = &mut module.content.as_mut().unwrap().1;

    let item_use = TokenStream::from(quote!(use gfp_core::derive::PhantomData;));
    let item_use: syn::ItemUse = syn::parse_macro_input!(item_use as _);
    contents.push(syn::Item::Use(item_use));

    let mut fields_marker = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new = syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

    let (generic_header, generic, where_clause) = generics.split_for_impl();
    for (i, field) in data.fields.iter().enumerate() {
        let ident = field.ident.as_ref().cloned()
            .unwrap_or_else(|| syn::Ident::new(&i.to_string(), proc_macro2::Span::call_site()));
        
        contents.push(item!(
            pub struct #ident<T>(PhantomData<T>);
        ));
        
        contents.push(item!(
            impl<T> #ident<T> {
                pub fn new() -> Self {
                    Self(PhantomData)
                }
            }
        ));

        let ty = &field.ty;
        
        contents.push(item!(
            unsafe impl #generic_header gfp_core::Field for #ident<super::#input_ident#generic> {
                type Parent = super::#input_ident#generic;
                type Type = #ty;

                fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type> {
                    unimplemented!()
                }
            }
        ));

        let ty = TokenStream::from(quote!(
            #module_name::#ident<#input_ident #generic>
        ));
        let ty = syn::parse_macro_input!(ty as syn::Type);

        let field_name = &field.ident;

        fields_new.push(expr!(
            #field_name: #module_name::#ident::new()
        ));

        let item = syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(syn::VisPublic { pub_token: syn::Token![pub](proc_macro2::Span::call_site()) }),
            ident: field.ident.clone(),
            colon_token: field.colon_token,
            ty
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
            pub fn fields(&self) -> #field_type_name #generic {
                #field_type_name {
                    #fields_new
                }
            }
        }
    })
}

fn derive_union(ty: syn::DeriveInput) -> TokenStream {
    unimplemented!("Unions are currently unsupported")
}

fn new_module(ident: syn::Ident) -> syn::ItemMod {
    syn::ItemMod {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mod_token: syn::Token![mod](proc_macro2::Span::call_site()),
        ident,
        content: Some((
            syn::token::Brace {
                span: proc_macro2::Span::call_site()
            },
            Vec::new()
        )),
        semi: None
    }
}