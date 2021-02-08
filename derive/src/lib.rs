extern crate proc_macro;

use proc_macro_roids::*;

use proc_macro::TokenStream;
use quote::quote;

/// This macro generates a number of field types and automatically derives
/// `gfp_core::Field` for them. It will also generate a type to make accessing
/// these field types easier.
///
/// The field types will be generated in a module named `{$type}_fields` and the
/// type that holds all of the field types will be called `{$type}::Fields`.
///
/// For `unions`, getting the field types is `unsafe` because you can cause
/// aliasing of unique references and because accessing union fields is
/// inherently `unsafe`.
///
/// For `struct`, getting the field types is safe since the memory operations
/// defined use raw pointers to initialized fields so UB is not possible.
///
///  * note: unit structs don't generate any extra code (i.e. `struct Foo;`)
///
/// `enums` are not supported.
///
/// For example for a struct,
/// ```
/// # #![feature(raw_ref_op)]
/// # mod test {
/// # use gfp_core::Field;
/// #[derive(Field)]
/// struct Person {
///     name: String,
///     age: u16,
///     children: Vec<Person>,
/// }
/// # }
/// ```
/// will generate (excluding comments) the following. Note the use of `&raw `,
/// this is to allow safe projection through raw pointers.
/// ```
/// # #![feature(raw_ref_op)]
/// # mod test {
/// struct Person {
///     name: String,
///     age: u16,
///     children: Vec<Person>,
/// }
/// // This struct holds all of the field types for easy access
/// struct PersonFields {
///     name: Person_fields::name<Person>,
///     age: Person_fields::age<Person>,
///     children: Person_fields::children<Person>,
/// }
/// impl Person {
///     const FIELDS: PersonFields = PersonFields {
///         name: Person_fields::name::INIT,
///         age: Person_fields::age::INIT,
///         children: Person_fields::children::INIT,
///     };
///
///     // get an instance of `PersonFields` easily by calling
///     // `Person::fields()`, then you can use this to access the
///     //
///     // ```rust
///     // let fields = Person::fields();
///     //
///     // person.project_to(fields.name);
///     // person.project_to(fields.age);
///     // person.project_to(fields.children);
///     // ```
///     fn fields() -> PersonFields {
///         PersonFields {
///             name: Person_fields::name::INIT,
///             age: Person_fields::age::INIT,
///             children: Person_fields::children::INIT,
///         }
///     }
/// }
/// #[allow(non_snake_case)]
/// mod Person_fields {
///     use super::*;
///     // represents the `name` field of `Person`
///     #[allow(non_camel_case_types)]
///     pub struct name<T>(::gfp_core::derive::Invariant<T>);
///     impl<T> name<T> {
///         pub const INIT: Self = Self(::gfp_core::derive::Invariant::INIT);
///     }
///     impl<T> Clone for name<T> {
///         fn clone(&self) -> Self {
///             *self
///         }
///     }
///     impl<T> Copy for name<T> {}
///     unsafe impl ::gfp_core::Field for name<super::Person> {
///         type Parent = super::Person;
///         type Type = String;
///         #[inline]
///         unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///             &raw const (*ptr).name
///         }
///         #[inline]
///         unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///             &raw mut (*ptr).name
///         }
///     }
///     // represents the `age` field of `Person`
///     #[allow(non_camel_case_types)]
///     pub struct age<T>(::gfp_core::derive::Invariant<T>);
///     impl<T> age<T> {
///         pub const INIT: Self = Self(::gfp_core::derive::Invariant::INIT);
///     }
///     impl<T> Clone for age<T> {
///         fn clone(&self) -> Self {
///             *self
///         }
///     }
///     impl<T> Copy for age<T> {}
///     unsafe impl ::gfp_core::Field for age<super::Person> {
///         type Parent = super::Person;
///         type Type = u16;
///         #[inline]
///         unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///             &raw const (*ptr).age
///         }
///         #[inline]
///         unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///             &raw mut (*ptr).age
///         }
///     }
///     // represents the `children` field of `Person`
///     #[allow(non_camel_case_types)]
///     pub struct children<T>(::gfp_core::derive::Invariant<T>);
///     impl<T> children<T> {
///         pub const INIT: Self = Self(::gfp_core::derive::Invariant::INIT);
///     }
///     impl<T> Clone for children<T> {
///         fn clone(&self) -> Self {
///             *self
///         }
///     }
///     impl<T> Copy for children<T> {}
///     unsafe impl ::gfp_core::Field for children<super::Person> {
///         type Parent = super::Person;
///         type Type = Vec<Person>;
///         #[inline]
///         unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///             &raw const (*ptr).children
///         }
///         #[inline]
///         unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///             &raw mut (*ptr).children
///         }
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Field)]
pub fn derive_field(ty: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(ty as syn::DeriveInput);

    match ty.data {
        syn::Data::Struct(_) => derive_struct(ty),
        syn::Data::Union(_) => derive_union(ty),
        syn::Data::Enum(_) => {
            syn::Error::new(ty.ident.span(), "enums are not supported")
                .to_compile_error()
                .into()
        },
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
        }) => {
            syn::Error::new(ty.ident.span(), "Unit structs are not supported")
                .to_compile_error()
                .into()
        },
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

    let mut fields_marker =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

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
                pub const INIT: Self = Self(::gfp_core::derive::Invariant::INIT);
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
            vis: field.vis,
            ident: Some(ident),
            colon_token: field.colon_token,
            ty,
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {
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

        #[allow(non_snake_case)]
        #module
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

    let mut fields_marker =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

    contents.push(item!(
        use super::*;
    ));

    let (generic_header, generic, where_clause) = generics.split_for_impl();
    for (i, field) in fields.unnamed.iter().enumerate() {
        use syn::spanned::Spanned;
        let ident = quote::format_ident!("_{}", i, span = field.span());

        contents.push(item!(
            #[allow(non_camel_case_types)]
            pub struct #ident<T>(::gfp_core::derive::Invariant<T>);
        ));

        contents.push(item!(
            impl<T> #ident<T> {
                pub const INIT: Self = Self(::gfp_core::derive::Invariant::INIT);
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
            span:  proc_macro2::Span::call_site(),
        });

        contents.push(item!(
            unsafe impl #generic_header ::gfp_core::Field for #ident<super::#input_ident #generic> {
                type Parent = super::#input_ident #generic;
                type Type = #ty;

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
            ty,
            ..field.clone()
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {
        struct #field_type_name #generic_header(#fields_marker) #where_clause;

        impl#generic_header #input_ident #generic #where_clause {
            const FIELDS: #field_type_name #generic = #field_type_name(#fields_new);

            fn fields() -> #field_type_name #generic {
                #field_type_name(#fields_new)
            }
        }

        #[allow(non_snake_case)]
        #module
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

    let fields = if let syn::Data::Union(syn::DataUnion {
        fields, ..
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

    let mut fields_marker =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();
    let mut fields_new =
        syn::punctuated::Punctuated::<_, syn::Token![,]>::new();

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
                    Self(::gfp_core::derive::Invariant::INIT)
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
            #ident: #module_name::#ident::init()
        ));

        let item = syn::Field {
            attrs: Vec::new(),
            vis: field.vis,
            ident: Some(ident),
            colon_token: field.colon_token,
            ty,
        };

        fields_marker.push(item);
    }

    let field_type_name = input_ident.append("Fields");

    TokenStream::from(quote! {
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

        #[allow(non_snake_case)]
        #module
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
