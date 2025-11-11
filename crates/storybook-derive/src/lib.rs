use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

// Helper to extract story attributes from a field
fn get_story_attrs(field: &syn::Field) -> (Option<String>, Option<String>) {
    let mut control_type = None;
    let mut default_value = None;
    
    for attr in &field.attrs {
        if attr.path().is_ident("story") {
            // Try parsing as a list of name-value pairs
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("control") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                            control_type = Some(lit_str.value());
                        }
                    }
                } else if meta.path.is_ident("default") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                            default_value = Some(lit_str.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }
    
    (control_type, default_value)
}

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
        let ty_string = quote!(#field_ty).to_string();
        let is_option = ty_string.starts_with("Option <");

        let (control_type, default_value) = get_story_attrs(field);

        let control = if let Some(control_type) = control_type {
            match control_type.as_str() {
                "color" => quote! { storybook::ControlType::Color },
                "select" => quote! { storybook::ControlType::Select },
                _ => quote! { storybook::ControlType::Text },
            }
        } else if ty_string.contains("bool") {
            quote! { storybook::ControlType::Boolean }
        } else if ty_string.contains("i32") || ty_string.contains("f32") || ty_string.contains("u32") || ty_string.contains("f64") {
            quote! { storybook::ControlType::Number }
        } else {
            quote! { storybook::ControlType::Text }
        };

        let default_value_quoted = match default_value {
            Some(v) => quote! { Some(#v.to_string()) },
            None => quote! { None },
        };

        quote! {
            storybook::ArgType {
                name: #field_name_str.to_string(),
                default_value: #default_value_quoted,
                control: #control,
                required: !#is_option,
            }
        }
    });

    let name_str = name.to_string();

    // Generate helper methods
    let expanded = quote! {
        impl #impl_generics storybook::StoryMeta for #name #ty_generics #where_clause {
            fn name() -> &'static str {
                #name_str
            }

            fn args() -> Vec<storybook::ArgType> {
                vec![
                    #(#arg_types),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for StorySelect trait
/// 
/// This macro generates select control options from an enum.
/// Each variant becomes an option in a select dropdown in Storybook.
/// Also implements FromStr for deserializing from Storybook values.
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

    // Generate FromStr match arms
    let from_str_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        
        quote! {
            #variant_str => Ok(#name::#variant_name)
        }
    });

    // Generate Display match arms
    let display_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        
        quote! {
            #name::#variant_name => #variant_str
        }
    });

    let name_str = name.to_string();

    // Generate implementation
    let expanded = quote! {
        impl #impl_generics storybook::StorySelect for #name #ty_generics #where_clause {
            fn type_name() -> &'static str {
                #name_str
            }

            fn options() -> Vec<String> {
                vec![
                    #(#options),*
                ]
            }
        }

        // Auto-register enum options on first use
        impl #impl_generics #name #ty_generics #where_clause {
            #[doc(hidden)]
            pub fn __register_enum_options() {
                storybook::register_enum_options(
                    #name_str,
                    <#name as storybook::StorySelect>::options()
                );
            }
        }

        impl #impl_generics std::str::FromStr for #name #ty_generics #where_clause {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms,)*
                    _ => Err(format!("Invalid {} variant: {}", #name_str, s))
                }
            }
        }

        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    #(#display_arms,)*
                };
                write!(f, "{}", s)
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
            storybook::register_story::<#ty>();
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

/// Macro to generate a registration function for all enums
/// Usage: register_enums!(AlertType, ButtonSize);
#[proc_macro]
pub fn register_enums(input: TokenStream) -> TokenStream {
    let types = syn::parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated);
    
    let registrations = types.iter().map(|ty| {
        quote! {
            #ty::__register_enum_options();
        }
    });
    
    let expanded = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn init_enums() {
            #(#registrations)*
        }
    };
    
    TokenStream::from(expanded)
}
