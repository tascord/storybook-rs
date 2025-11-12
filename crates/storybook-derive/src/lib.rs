use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

// Helper to extract story attributes from a field
// Returns: (control_type, default_value, from_type, lorem_word_count)
fn get_story_attrs(field: &syn::Field) -> (Option<String>, Option<String>, Option<syn::Type>, Option<usize>) {
    let mut control_type = None;
    let mut default_value = None;
    let mut from_type = None;
    let mut lorem_count = None;

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
                } else if meta.path.is_ident("from") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                            from_type =
                                Some(syn::parse_str(&lit_str.value()).expect("Invalid type for from"));
                        }
                    }
                } else if meta.path.is_ident("lorem") {
                    // Handle both `#[story(lorem)]` (defaults to 8) and `#[story(lorem = "N")]`
                    if let Ok(value) = meta.value() {
                        if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                            if let Ok(count) = lit_str.value().parse::<usize>() {
                                lorem_count = Some(count);
                            }
                        }
                    } else {
                        // No value specified, use default of 8
                        lorem_count = Some(8);
                    }
                }
                Ok(())
            });
        }
    }

    (control_type, default_value, from_type, lorem_count)
}

// Generate lorem ipsum text with specified number of words
fn generate_lorem_ipsum(word_count: usize) -> String {
    const LOREM_WORDS: &[&str] = &[
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
        "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et",
        "dolore", "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis",
        "nostrud", "exercitation", "ullamco", "laboris", "nisi", "aliquip", "ex", "ea",
        "commodo", "consequat", "duis", "aute", "irure", "in", "reprehenderit", "voluptate",
        "velit", "esse", "cillum", "fugiat", "nulla", "pariatur", "excepteur", "sint",
        "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui", "officia",
        "deserunt", "mollit", "anim", "id", "est", "laborum", "pellentesque", "habitant",
        "morbi", "tristique", "senectus", "netus", "et", "malesuada", "fames", "ac",
        "turpis", "egestas", "vestibulum", "tortor", "quam", "feugiat", "vitae", "ultricies",
        "legimus", "typi", "qui", "nusquam", "vici", "sunt", "signa", "consuetudium"
    ];
    
    let mut words = Vec::new();
    for i in 0..word_count {
        words.push(LOREM_WORDS[i % LOREM_WORDS.len()]);
    }
    words.join(" ")
}


fn generate_storybook_js(name: &str, _fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, arg_types: &[(String, String, String, String, String)]) {
    // Generate argTypes from fields
    let arg_types_json: Vec<String> = arg_types.iter().map(|(field_name, control, _default_val, required, options_json)| {
        let options_str = if !options_json.is_empty() {
            format!(", options: {}", options_json)
        } else {
            String::new()
        };
        
        let required_str = if required == "true" {
            ", table: { category: 'required' }"
        } else {
            ""
        };
        
        format!(
            "    {}: {{\n      control: '{}',\n      description: '{}'{}{}\n    }}",
            field_name, control, field_name, options_str, required_str
        )
    }).collect();
    
    let args_str = arg_types_json.join(",\n");
    
    // Generate default args
    let default_args: Vec<String> = arg_types.iter().map(|(field_name, _, default_val, _, _)| {
        format!("  {}: {}", field_name, default_val)
    }).collect();
    
    let default_args_str = default_args.join(",\n");
    
    let js_content = format!(r#"import init, {{ register_all_stories, render_story, get_enum_options, init_enums }} from '../../example/pkg/example.js';

// Initialize WASM
await init();

console.log('About to call init_enums...');
init_enums();
console.log('init_enums called');

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
"#, name, args_str, name, default_args_str);

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

#[proc_macro_derive(Story, attributes(story))]
pub fn derive_story(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let name_str = name.to_string();
    let story_args_name = syn::Ident::new(&format!("{}StoryArgs", name), name.span());

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Story can only be derived for structs with named fields"),
        },
        _ => panic!("Story can only be derived for structs"),
    };

    let story_args_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let (control_type, _, from_type, _) = get_story_attrs(field);
        
        // Make select control fields optional so they can deserialize from undefined
        let should_be_optional = control_type.as_ref().map(|c| c == "select").unwrap_or(false);

        if let Some(from_type) = from_type {
            if should_be_optional {
                quote! {
                    #[serde(default)]
                    pub #field_name: Option<#from_type>
                }
            } else {
                quote! {
                    #[serde(default)]
                    pub #field_name: #from_type
                }
            }
        } else {
            if should_be_optional {
                quote! {
                    #[serde(default)]
                    pub #field_name: Option<#field_ty>
                }
            } else {
                quote! {
                    #[serde(default)]
                    pub #field_name: #field_ty
                }
            }
        }
    });

    let from_impl_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let (control_type, _, _, _) = get_story_attrs(field);
        let should_be_optional = control_type.as_ref().map(|c| c == "select").unwrap_or(false);
        
        if should_be_optional {
            // For optional enum fields, unwrap_or_default() or just use the option as-is
            quote! { #field_name: value.#field_name.unwrap_or_default() }
        } else {
            quote! { #field_name: value.#field_name.into() }
        }
    });

    // Generate arg type information for each field
    let mut arg_types_for_js: Vec<(String, String, String, String, String)> = Vec::new();
    
    let arg_types = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let field_ty = &field.ty;
        let ty_string = quote!(#field_ty).to_string();
        let is_option = ty_string.starts_with("Option <");

        let (control_type, default_value, from_type, lorem_count) = get_story_attrs(field);

        let mut options = quote! { None };
        let mut options_json = String::new();
        let control = if let Some(ref control_type) = control_type {
            match control_type.as_str() {
                "color" => quote! { storybook::ControlType::Color },
                "select" => {
                    options = quote! { Some(<#field_ty as storybook::StorySelect>::options()) };
                    // Extract the enum type name from the field type
                    let enum_type_name = ty_string.trim().replace(" ", "");
                    options_json = format!("get_enum_options('{}')", enum_type_name);
                    quote! { storybook::ControlType::Select }
                }
                _ => quote! { storybook::ControlType::Text },
            }
        } else {
            let ty_to_check = if let Some(from_type) = &from_type {
                quote!(#from_type).to_string()
            } else {
                ty_string.clone()
            };

            if ty_to_check.contains("bool") {
                quote! { storybook::ControlType::Boolean }
            } else if ty_to_check.contains("i32")
                || ty_to_check.contains("f32")
                || ty_to_check.contains("u32")
                || ty_to_check.contains("f64")
                || ty_to_check.contains("usize")
            {
                quote! { storybook::ControlType::Number }
            } else {
                quote! { storybook::ControlType::Text }
            }
        };

        let default_value_quoted = match &default_value {
            Some(v) => quote! { Some(#v.to_string()) },
            None => {
                if let Some(lorem_word_count) = lorem_count {
                    let lorem_text = generate_lorem_ipsum(lorem_word_count);
                    quote! { Some(#lorem_text.to_string()) }
                } else {
                    quote! { None }
                }
            }
        };
        
        let control_str = match control_type.as_ref() {
            Some(ct) => {
                match ct.as_str() {
                    "color" => "color".to_string(),
                    "select" => "select".to_string(),
                    _ => "text".to_string(),
                }
            }
            None => {
                if ty_string.contains("bool") {
                    "boolean".to_string()
                } else if ty_string.contains("i32") || ty_string.contains("f32") || ty_string.contains("u32") || ty_string.contains("f64") || ty_string.contains("usize") {
                    "number".to_string()
                } else {
                    "text".to_string()
                }
            }
        };
        
        let default_val_str = match &default_value {
            Some(dv) => dv.clone(),
            None => {
                if let Some(lorem_word_count) = lorem_count {
                    // Generate lorem ipsum text
                    format!("'{}'", generate_lorem_ipsum(lorem_word_count))
                } else if control_str == "select" {
                    "null".to_string()
                } else if ty_string.contains("String") {
                    "''".to_string()
                } else if ty_string.contains("bool") {
                    "false".to_string()
                } else if ty_string.contains("i32") || ty_string.contains("f32") || ty_string.contains("u32") || ty_string.contains("f64") || ty_string.contains("usize") {
                    "0".to_string()
                } else {
                    "undefined".to_string()
                }
            }
        };
        
        arg_types_for_js.push((
            field_name_str.clone(),
            control_str,
            default_val_str,
            if is_option { "false" } else { "true" }.to_string(),
            options_json,
        ));

        quote! {
            storybook::ArgType {
                name: #field_name_str.to_string(),
                default_value: #default_value_quoted,
                control: #control,
                required: !#is_option,
                options: #options,
            }
        }
    }).collect::<Vec<_>>();

    // Generate the Storybook JavaScript file
    generate_storybook_js(&name_str, fields, &arg_types_for_js);

    // Generate helper methods
    let expanded = quote! {
        #[derive(serde::Deserialize, Default)]
        pub struct #story_args_name {
            #(#story_args_fields),*
        }

        impl From<#story_args_name> for #name {
            fn from(value: #story_args_name) -> Self {
                Self {
                    #(#from_impl_fields),*
                }
            }
        }

        impl #impl_generics storybook::StoryMeta for #name #ty_generics #where_clause {
            type StoryArgs = #story_args_name;

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
