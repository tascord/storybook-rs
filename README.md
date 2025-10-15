# storybook-rs

A tool for rendering Rust-generated web components in Storybook using `dominator` + `wasm-pack`. This library provides derive macros and automatic component registration to create an interactive component viewer.

## Features

- 🦀 **Rust-first**: Write web components in Rust using the `dominator` library
- 🎨 **Derive Macros**: Automatically generate component metadata with `#[derive(Story)]`
- 📦 **WASM-powered**: Compile to WebAssembly with `wasm-pack`
- 🔍 **Auto-discovery**: Components are automatically registered and discoverable
- 🎭 **Interactive Preview**: Built-in component viewer with live controls

## Project Structure

```
storybook-rs/
├── storybook-core/      # Core library with Story trait and runtime
├── storybook-derive/    # Procedural macros for deriving Story
├── example/             # Example components (Button, Card, Input)
└── storybook/           # Web-based component viewer
```

## Quick Start

### 1. Define a Component

```rust
use storybook_core::{Story, register_story, StoryMeta};
use storybook_derive::Story as DeriveStory;
use dominator::{Dom, html};
use wasm_bindgen::prelude::*;
use serde::Deserialize;

#[derive(DeriveStory, Deserialize)]
pub struct Button {
    pub label: String,
    pub color: String,
}

impl Story for Button {
    fn name() -> &'static str {
        Button::story_name()
    }

    fn args() -> Vec<storybook_core::ArgType> {
        Button::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let button: Button = serde_wasm_bindgen::from_value(args)
            .unwrap_or(Button {
                label: "Click me".to_string(),
                color: "#007bff".to_string(),
            });
        
        html!("button", {
            .text(&button.label)
            .style("background-color", &button.color)
            .style("color", "white")
            .style("border", "none")
            .style("padding", "10px 20px")
            .style("border-radius", "4px")
            .style("cursor", "pointer")
        })
    }
}
```

### 2. Register Components

```rust
#[wasm_bindgen]
pub fn register_all_stories() {
    register_story(StoryMeta {
        name: Button::name(),
        args: Button::args,
        render_fn: Button::render,
    });
}
```

### 3. Build with wasm-pack

```bash
cd example
wasm-pack build --target web --out-dir pkg
```

### 4. View Components

Open `storybook/index.html` in a web browser or serve it with a local server:

```bash
python3 -m http.server 8000
# Navigate to http://localhost:8000/storybook/
```

## How It Works

1. **Derive Macro**: The `#[derive(Story)]` macro inspects struct fields and generates metadata about component arguments
2. **Registration**: Components are registered at runtime in a global registry
3. **Rendering**: The viewer calls `render_story(name, args)` which deserializes arguments and renders the component
4. **DOM Integration**: `dominator` efficiently manages the DOM and applies your component's styling

## Example Components

The repository includes three example components:

- **Button**: A styled button with customizable label and color
- **Card**: A card component with title, content, and background color
- **Input**: A text input with placeholder and value

## Development

```bash
# Check all crates
cargo check

# Build WASM package
cd example && wasm-pack build --target web --out-dir pkg

# Run the viewer
cd .. && python3 -m http.server 8000
```

## Architecture

- **storybook-core**: Provides the `Story` trait, `StoryMeta` type, and runtime functions (`get_stories`, `render_story`)
- **storybook-derive**: Procedural macro that generates `story_name()` and `story_args()` helper methods
- **Components**: Implement the `Story` trait and provide a `render()` method that returns a `Dom` node

## License

MIT
