use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for Story trait
/// 
/// This macro automatically generates helper implementations for the Story trait,
/// extracting field information to generate ArgTypes for Storybook.
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
                control: storybook_core::ControlType::Text,
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
            
            /// Register this story with the global registry
            pub fn register() {
                storybook_core::register_story(storybook_core::StoryMeta {
                    name: #name::name(),
                    args: #name::args,
                    render_fn: #name::render,
                });
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for StorySelect trait
/// 
/// This macro generates select control options from an enum.
/// Each variant becomes an option in a select dropdown in Storybook.
#[proc_macro_derive(StorySelect, attributes(story_select))]
pub fn derive_story_select(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract variant information
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("StorySelect can only be derived for enums"),
    };

    // Generate option values from enum variants
    let options = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        
        quote! {
            #variant_str.to_string()
        }
    });

    let name_str = name.to_string();

    // Generate implementation
    let expanded = quote! {
        impl #impl_generics storybook_core::StorySelect for #name #ty_generics #where_clause {
            fn type_name() -> &'static str {
                #name_str
            }

            fn options() -> Vec<String> {
                vec![
                    #(#options),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro to generate a registration function for all stories
/// Usage: register_stories!(Button, Card, Input);
#[proc_macro]
pub fn register_stories(input: TokenStream) -> TokenStream {
    let types = syn::parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated);
    
    let registrations = types.iter().map(|ty| {
        quote! {
            #ty::register();
        }
    });
    
    let expanded = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn register_all_stories() {
            #(#registrations)*
        }
    };
    
    TokenStream::from(expanded)
}
