# Usage Guide

## Creating a New Component

### 1. Define the Component Struct

```rust
use storybook_core::Story;
use storybook_derive::Story as DeriveStory;
use dominator::{Dom, html};
use wasm_bindgen::prelude::*;
use serde::Deserialize;

#[derive(DeriveStory, Deserialize)]
pub struct MyComponent {
    pub text: String,
    pub size: String,
}
```

### 2. Implement the Story Trait

```rust
impl Story for MyComponent {
    fn name() -> &'static str {
        MyComponent::story_name()
    }

    fn args() -> Vec<storybook_core::ArgType> {
        MyComponent::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let component: MyComponent = serde_wasm_bindgen::from_value(args)
            .unwrap_or(MyComponent {
                text: "Hello".to_string(),
                size: "16px".to_string(),
            });
        
        html!("div", {
            .text(&component.text)
            .style("font-size", &component.size)
        })
    }
}
```

### 3. Register the Component

```rust
#[wasm_bindgen]
pub fn register_all_stories() {
    register_story(StoryMeta {
        name: MyComponent::name(),
        args: MyComponent::args,
        render_fn: MyComponent::render,
    });
}
```

## Building

```bash
# Build the WASM package
cd example
wasm-pack build --target web --out-dir pkg

# Return to root
cd ..
```

## Running the Viewer

```bash
# Start a local server
python3 -m http.server 8000

# Open in browser
# Navigate to http://localhost:8000/storybook/
```

## Component Structure

### Required Traits

- `Story`: Main trait for components
- `Deserialize`: For deserializing arguments from JavaScript

### Derive Macro

The `#[derive(Story)]` macro automatically generates:
- `story_name()`: Returns the component name
- `story_args()`: Returns the list of arguments with types

### Rendering

The `render()` method must:
1. Deserialize args from `JsValue`
2. Provide fallback values
3. Return a `Dom` node using dominator's `html!` macro

## Dominator Tips

### Creating Elements

```rust
html!("div", {
    .text("Hello")
    .style("color", "blue")
})
```

### Nesting Elements

```rust
html!("div", {
    .children(&mut [
        html!("h1", { .text("Title") }),
        html!("p", { .text("Content") }),
    ])
})
```

### Attributes

```rust
html!("input" => web_sys::HtmlInputElement, {
    .attr("type", "text")
    .attr("placeholder", "Enter text...")
})
```

## Troubleshooting

### WASM Build Fails

Make sure you have wasm-pack installed:
```bash
cargo install wasm-pack
```

### Components Don't Appear

1. Check that `register_all_stories()` is being called
2. Verify the WASM module is loading in browser console
3. Ensure the component implements all required trait methods

### Styling Issues

Remember to use inline styles via `.style()` or add global CSS in the HTML file.
