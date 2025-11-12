use wasm_bindgen::prelude::*;
use dominator::{Dom, html};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Re-export for use in derive macro
pub use storybook_derive::{register_stories, Story as StoryDerive, StorySelect, register_enums};

/// Control type for Storybook args
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlType {
    Text,
    Select,
    Color,
    Boolean,
    Number,
}

/// Argument type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgType {
    pub name: String,
    pub default_value: Option<String>,
    pub control: ControlType,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

/// Story trait that components must implement
///
/// Components can implement this trait and return any type that converts to Dom.
/// This allows using dominator's builder patterns naturally.
pub trait Story: 'static + Sized {
    fn to_story(self) -> Dom;
}

/// Trait for story metadata, to be implemented by the derive macro
pub trait StoryMeta: Sized {
    type StoryArgs: for<'de> Deserialize<'de> + Into<Self>;
    fn name() -> &'static str;
    fn args() -> Vec<ArgType>;
}

/// Extension trait for types that can be converted to stories
/// 
/// This trait allows types to be used as stories by implementing
/// a simple `to_dom()` method that returns a Dom node.
pub trait IntoStory {
    /// Convert this type into a Dom node
    fn to_story(self) -> Dom;
}

/// Blanket implementation for types that already are Dom
impl IntoStory for Dom {
    fn to_story(self) -> Dom {
        self
    }
}

/// StorySelect trait for enums that should appear as select controls
pub trait StorySelect: 'static {
    /// Get the enum type name
    fn type_name() -> &'static str;
    
    /// Get all possible values as strings
    fn options() -> Vec<String>;
}

/// Story metadata for registration
pub struct StoryRegistration {
    pub name: &'static str,
    pub args: fn() -> Vec<ArgType>,
    pub render_fn: fn(JsValue) -> Dom,
}

unsafe impl Sync for StoryRegistration {}

// Global registry for stories
static STORY_REGISTRY: Lazy<Mutex<Vec<StoryRegistration>>> = Lazy::new(|| Mutex::new(Vec::new()));

// Global registry for enum options
static ENUM_REGISTRY: Lazy<Mutex<std::collections::HashMap<String, Vec<String>>>> = 
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

/// Register a story with the global registry
#[doc(hidden)]
pub fn register_story<T: Story + StoryMeta>() {
    let registration = StoryRegistration {
        name: T::name(),
        args: T::args,
        render_fn: |args: JsValue| {
            let component: T::StoryArgs = serde_wasm_bindgen::from_value(args).unwrap();
            let story: T = component.into();
            story.to_story()
        },
    };
    STORY_REGISTRY.lock().unwrap().push(registration);
}

/// Register an enum's options with the global registry
#[doc(hidden)]
pub fn register_enum_options(type_name: &'static str, options: Vec<String>) {
    web_sys::console::log_1(&format!("Registering enum {}: {:?}", type_name, options).into());
    ENUM_REGISTRY.lock().unwrap().insert(type_name.to_string(), options);
}

/// Get enum options for a given type name
#[wasm_bindgen]
pub fn get_enum_options(type_name: &str) -> JsValue {
    let registry = ENUM_REGISTRY.lock().unwrap();
    web_sys::console::log_1(&format!("Getting enum options for {}, registry has {} entries", type_name, registry.len()).into());
    if let Some(options) = registry.get(type_name) {
        web_sys::console::log_1(&format!("Found options: {:?}", options).into());
        serde_wasm_bindgen::to_value(options).unwrap_or(JsValue::NULL)
    } else {
        web_sys::console::log_1(&format!("No options found for {}", type_name).into());
        JsValue::NULL
    }
}

/// Macro to help register stories - used by derive macro
#[macro_export]
macro_rules! __register_story {
    ($ty:ty) => {{
        $crate::register_story($crate::StoryMeta {
            name: <$ty as $crate::Story>::name(),
            args: <$ty as $crate::Story>::args,
            render_fn: <$ty as $crate::Story>::render,
        });
    }};
}

/// Get all registered stories as Storybook-compatible format
#[wasm_bindgen]
pub fn get_stories() -> JsValue {
    let stories = STORY_REGISTRY.lock().unwrap();
    let story_defs: Vec<_> = stories
        .iter()
        .map(|meta| {
            let args = (meta.args)();
            let mut arg_types = serde_json::Map::new();
            let mut default_args = serde_json::Map::new();

            for arg in args {
                let control = serde_json::to_value(&arg.control).unwrap();

                let mut table = std::collections::HashMap::new();
                if arg.required {
                    table.insert("category".to_string(), "required".to_string());
                } else {
                    table.insert("category".to_string(), "optional".to_string());
                }

                let mut arg_map = serde_json::Map::new();
                arg_map.insert("name".to_string(), serde_json::Value::String(arg.name.clone()));
                arg_map.insert("control".to_string(), control);
                arg_map.insert("table".to_string(), serde_json::to_value(table).unwrap());

                if let Some(default) = arg.default_value {
                    default_args.insert(arg.name.clone(), serde_json::Value::String(default));
                }

                arg_types.insert(arg.name, serde_json::Value::Object(arg_map));
            }

            serde_json::json!({
                "name": meta.name,
                "argTypes": arg_types,
                "args": default_args,
            })
        })
        .collect();

    serde_wasm_bindgen::to_value(&story_defs).unwrap()
}

/// Render a story by name with the given arguments
/// Returns the DOM node for the story
#[wasm_bindgen]
pub fn render_story(name: &str, args: JsValue) -> Result<web_sys::Node, JsValue> {
    let story_dom = STORY_REGISTRY
        .lock()
        .unwrap()
        .iter()
        .find(|meta| meta.name == name)
        .map(|meta| (meta.render_fn)(args.clone()))
        .ok_or_else(|| JsValue::from_str(&format!("Story '{}' not found", name)))?;
    
    // Create a container element
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
    let container = document.create_element("div")?;
    
    // Append the story DOM to the container
    dominator::append_dom(&container, story_dom);
    
    // Return the container as a Node
    Ok(container.into())
}

/// Export stories in Storybook CSF (Component Story Format) compatible format
#[wasm_bindgen]
pub fn export_stories_csf() -> JsValue {
    get_stories()
}

/// Initialize the storybook runtime
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        web_sys::console::error_1(&JsValue::from_str(&msg));
    }));
}

/// Example helper for creating a simple text component
pub fn text_component(content: &str) -> Dom {
    html!("div", {
        .text(content)
    })
}

/// Example helper for creating a styled component
pub fn styled_component(content: &str, color: &str) -> Dom {
    html!("div", {
        .text(content)
        .style("color", color)
        .style("padding", "10px")
        .style("border", "1px solid #ccc")
        .style("border-radius", "4px")
    })
}
