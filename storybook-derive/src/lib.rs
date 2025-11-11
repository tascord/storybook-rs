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

// Helper to extract control type (backwards compatibility)
fn get_control_type(field: &syn::Field) -> Option<String> {
    get_story_attrs(field).0
}

// Helper to extract default value from story attribute
fn get_default_value(field: &syn::Field) -> Option<String> {
    get_story_attrs(field).1
}

fn generate_storybook_js(name: &str, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) {
    // Collect enum type names for select controls
    let enum_types: Vec<String> = fields.iter()
        .filter(|f| get_control_type(f).as_deref() == Some("select"))
        .filter_map(|f| {
            // Get just the type, not attributes
            let ty = &f.ty;
            // Convert to string and clean it up
            let ty_str = quote!(#ty).to_string();
            let clean = ty_str.trim().replace(" ", "");
            if !clean.is_empty() {
                Some(clean)
            } else {
                None
            }
        })
        .collect();
    
    // Generate argTypes from fields
    let arg_types: Vec<String> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ty = &field.ty;
        let ty_str = quote!(#field_ty).to_string();
        
        // Check if field is Option<T>
        let is_optional = ty_str.contains("Option") && ty_str.contains("<");
        let table_required = if is_optional { 
            "" 
        } else { 
            ",\n      table: { category: 'required' }" 
        };
        
        // Check for explicit control type in attribute
        let control = if let Some(control_type) = get_control_type(field) {
            match control_type.as_str() {
                "color" => "control: 'color'".to_string(),
                "select" => {
                    // Hardcode options inline with proper Storybook format
                    "control: { type: 'select' }, options: ['Info', 'Success', 'Warning', 'Error']".to_string()
                },
                "range" => "control: { type: 'range', min: 0, max: 100, step: 1 }".to_string(),
                "boolean" => "control: 'boolean'".to_string(),
                "number" => "control: 'number'".to_string(),
                "text" => "control: 'text'".to_string(),
                other => format!("control: '{}'", other),
            }
        } else {
            // Auto-detect from type (strip Option< if present)
            let inner_ty = if is_optional {
                ty_str.replace("Option", "").replace("<", "").replace(">", "").trim().to_string()
            } else {
                ty_str.clone()
            };
            
            if inner_ty.contains("String") || inner_ty == "& str" {
                "control: 'text'".to_string()
            } else if inner_ty.contains("bool") {
                "control: 'boolean'".to_string()
            } else if inner_ty.contains("i32") || inner_ty.contains("u32") || inner_ty.contains("f32") || inner_ty.contains("f64") {
                "control: 'number'".to_string()
            } else {
                "control: 'text'".to_string()
            }
        };
        
        format!("    {}: {{\n      {},\n      description: '{}'{}\n    }}", 
            field_name, control, field_name, table_required)
    }).collect();
    
    let args_str = arg_types.join(",\n");
    
    // Generate default args
    let default_args: Vec<String> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ty = &field.ty;
        let ty_str = quote!(#field_ty).to_string();
        
        let control_type = get_control_type(field);
        let is_optional = ty_str.contains("Option") && ty_str.contains("<");
        
        // Check for explicit default value
        let default_val = if let Some(default) = get_default_value(field) {
            default
        } else if is_optional {
            // Optional fields default to undefined
            "undefined".to_string()
        } else if control_type.as_deref() == Some("color") {
            "'#000000'".to_string()
        } else if control_type.as_deref() == Some("select") {
            // For select controls, use first option
            if ty_str.contains("AlertType") {
                "'Info'".to_string()
            } else {
                "'default'".to_string()
            }
        } else if ty_str.contains("String") || ty_str == "& str" {
            format!("'{}'", field_name)
        } else if ty_str.contains("bool") {
            "false".to_string()
        } else if ty_str.contains("i32") || ty_str.contains("u32") || ty_str.contains("f32") || ty_str.contains("f64") {
            "0".to_string()
        } else {
            format!("'{}'", field_name)
        };
        
        format!("  {}: {}", field_name, default_val)
    }).collect();
    
    // Generate enum loading code for each enum type
    let enum_loading = enum_types.iter().map(|type_name| {
        let var_name = format!("{}Options", type_name.to_lowercase());
        format!("const {} = get_enum_options('{}') || [];\nconsole.log('Loaded {} options:', {});", var_name, type_name, type_name, var_name)
    }).collect::<Vec<_>>().join("\n");
    
    let default_args_str = default_args.join(",\n");
    
    let js_content = format!(r#"import init, {{ register_all_stories, render_story, get_enum_options, init_enums }} from '../../example/pkg/example.js';

// Initialize WASM
await init();

console.log('About to call init_enums...');
init_enums();
console.log('init_enums called');

// Load enum options for this component
{}

register_all_stories();

// Define the story with populated enum options
export default {{
  title: 'Components/{}',
  argTypes: {{
{}
  }},
}};

const Template = (args) => {{
  const container = document.createElement('div');
  const dom = render_story('{}', args);
  container.appendChild(dom);
  return container;
}};

export const Default = Template.bind({{}});
Default.args = {{
{}
}};
"#, enum_loading, name, args_str, name, default_args_str);

    // Write to storybook/stories directory
    let output_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| std::path::PathBuf::from(d).parent().unwrap().join("storybook/stories"))
        .unwrap_or_else(|_| std::path::PathBuf::from("storybook/stories"));
    
    if let Err(_) = std::fs::create_dir_all(&output_dir) {
        // Directory might already exist, that's fine
    }
    
    let output_file = output_dir.join(format!("{}.stories.js", name));
    let _ = std::fs::write(output_file, js_content);
}

/// Derive macro for Story trait
/// 
/// This macro automatically generates helper implementations for the Story trait,
/// extracting field information to generate ArgTypes for Storybook.
/// 
/// Components should implement an `into_dom(self) -> Dom` method to leverage
/// dominator's builder patterns naturally. The Story trait's `render()` method
/// can then simply deserialize and call `into_dom()`.
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

    // Generate the Storybook JavaScript file
    let name_str = name.to_string();
    generate_storybook_js(&name_str, fields);

    // Generate arg type information for each field
    let arg_types = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let field_ty = &field.ty;
        
        quote! {
            storybook::ArgType {
                name: #field_name_str.to_string(),
                ty: std::any::type_name::<#field_ty>().to_string(),
                control: storybook::ControlType::Text,
            }
        }
    });

    // Generate helper methods
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const fn story_name() -> &'static str {
                #name_str
            }

            pub fn story_args() -> Vec<storybook::ArgType> {
                vec![
                    #(#arg_types),*
                ]
            }
            
            /// Register this story with the global registry
            pub fn register() {
                storybook::register_story(storybook::StoryMeta {
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
