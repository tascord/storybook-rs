use wasm_bindgen::prelude::*;
use dominator::{Dom, html};
use serde::{Deserialize, Serialize};
use std::any::TypeId;

// Re-export for use in derive macro
pub use linkme;
pub use storybook_derive::Story;

/// Argument type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgType {
    pub name: String,
    pub ty: String,
}

/// Story trait that components must implement
pub trait Story: 'static + Sync {
    /// Get the story name
    fn name() -> &'static str;
    
    /// Get argument types for this story
    fn args() -> Vec<ArgType>;
    
    /// Render the component with given args
    fn render(args: JsValue) -> Dom;
}

/// Story metadata for registration
pub struct StoryMeta {
    pub name: &'static str,
    pub args: fn() -> Vec<ArgType>,
    pub type_id: TypeId,
    pub render_fn: fn(JsValue) -> Dom,
}

unsafe impl Sync for StoryMeta {}

// Collect all registered stories using linkme
#[linkme::distributed_slice]
pub static STORIES: [StoryMeta] = [..];

/// Get all registered stories
#[wasm_bindgen]
pub fn get_stories() -> JsValue {
    let stories: Vec<_> = STORIES
        .iter()
        .map(|meta| {
            serde_json::json!({
                "name": meta.name,
                "args": (meta.args)(),
            })
        })
        .collect();
    
    serde_wasm_bindgen::to_value(&stories).unwrap()
}

/// Render a story by name with the given arguments
#[wasm_bindgen]
pub fn render_story(name: &str, args: JsValue) -> Result<(), JsValue> {
    let story = STORIES
        .iter()
        .find(|meta| meta.name == name)
        .ok_or_else(|| JsValue::from_str(&format!("Story '{}' not found", name)))?;
    
    let dom = (story.render_fn)(args);
    
    // Get the root element
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
    let root = document
        .get_element_by_id("storybook-root")
        .ok_or_else(|| JsValue::from_str("No #storybook-root element found"))?;
    
    // Clear existing content
    root.set_inner_html("");
    
    // Append the new DOM
    dominator::append_dom(&root, dom);
    
    Ok(())
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
