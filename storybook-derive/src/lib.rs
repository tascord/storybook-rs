use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for Story trait
/// 
/// This macro automatically generates helper implementations for the Story trait,
/// extracting field information to generate ArgTypes for Storybook.
/// You still need to implement the render() method manually.
#[proc_macro_derive(Story, attributes(story))]
pub fn derive_story(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Story can only be derived for structs with named fields"),
        },
        _ => panic!("Story can only be derived for structs"),
    };

    // Generate arg type information for each field
    let arg_types = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let field_ty = &field.ty;
        
        quote! {
            storybook_core::ArgType {
                name: #field_name_str.to_string(),
                ty: std::any::type_name::<#field_ty>().to_string(),
            }
        }
    });

    let name_str = name.to_string();

    // Generate helper methods
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const fn story_name() -> &'static str {
                #name_str
            }

            pub fn story_args() -> Vec<storybook_core::ArgType> {
                vec![
                    #(#arg_types),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}
