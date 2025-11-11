# storybook

Storybook integration for Rust WebAssembly components using dominator.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
storybook = "0.1"
serde = { version = "1.0", features = ["derive"] }
dominator = "0.5"
wasm-bindgen = "0.2"
```

## Usage

1. **Add the derive macro to your component:**

```rust
use storybook_rs::{Story, StorySelect};
use dominator::Dom;
use serde::Deserialize;

#[derive(Story, Deserialize)]
pub struct Button {
    pub label: String,
    #[story(control = "color")]
    pub color: String,
}

impl Button {
    pub fn into_dom(self) -> Dom {
        // Your dominator component implementation
    }
}
```

2. **For enum types, use `StorySelect`:**

```rust
#[derive(StorySelect, Deserialize, Clone, Debug)]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Story, Deserialize)]
pub struct Alert {
    pub message: String,
    #[story(control = "select")]
    pub alert_type: AlertType,
}
```

The `StorySelect` derive automatically implements `FromStr` and `Display` for your enum.

3. **Available control attributes:**

- `#[story(control = "color")]` - Color picker
- `#[story(control = "select")]` - Dropdown select (for enums)
- `#[story(control = "range")]` - Range slider
- `#[story(control = "boolean")]` - Toggle
- `#[story(control = "number")]` - Number input
- `#[story(control = "text")]` - Text input (default for strings)

4. **Custom default values:**

You can specify custom default values for Storybook controls:

```rust
#[derive(Story, Deserialize)]
pub struct Button {
    #[story(default = "'Click Me!'")]
    pub label: String,
    #[story(control = "color", default = "'#007bff'")]
    pub color: String,
}
```

5. **Optional fields:**

Fields with `Option<T>` type are automatically marked as optional in Storybook:

```rust
#[derive(Story, Deserialize)]
pub struct Button {
    pub label: String,           // Required field
    pub disabled: Option<bool>,  // Optional field (defaults to undefined)
}
```

6. **Build your WASM component:**

```bash
cd example
cargo build  # Automatically generates .stories.js files
wasm-pack build --target web
```

The `#[derive(Story)]` macro automatically generates corresponding `.stories.js` files in `storybook/stories/` during compilation.

7. **Run Storybook:**

```bash
npm run storybook
```

That's it! No manual JavaScript needed - the macro handles it all.

## License

MIT
