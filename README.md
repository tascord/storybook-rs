# storybook

Storybook integration for Rust WebAssembly components using dominator.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
storybook = "0.2"
serde = { version = "1.0", features = ["derive"] }
dominator = "0.5"
wasm-bindgen = "0.2"
```

## Usage

1. **Define your component with the derive macro:**

```rust
use storybook::Story;
use dominator::Dom;
use serde::Deserialize;

#[derive(StoryDerive, Deserialize)]
pub struct Button {
    pub label: String,
    #[story(control = "color", default = "'#007bff'")]
    pub color: String,
}

impl Story for Button {
    fn to_story(self) -> Dom {
        // Your dominator component implementation
    }
}
```

2. **For enums, use `StorySelect`:**

```rust
#[derive(StorySelect, Deserialize, Clone, Debug, Default)]
pub enum AlertType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(StoryDerive, Deserialize)]
pub struct Alert {
    #[story(lorem = "5")]
    pub message: String,
    #[story(control = "select")]
    pub alert_type: AlertType,
}
```

3. **Field attributes:**

- `#[story(control = "color")]` - Color picker
- `#[story(control = "select")]` - Dropdown (for enums, auto-defaults to first variant)
- `#[story(default = "'value'")]` - Custom default value
- `#[story(from = "usize")]` - Type conversion via `From` trait
- `#[story(lorem = "N")]` - Auto-generate N words of lorem ipsum (defaults to 8 if no N)
- `#[story(skip)]` - Skip field in Storybook (useful for callbacks, closures, etc.)

4. **Register components:**

```rust
storybook::register_stories!(Button, Alert);
storybook::register_enums!(AlertType);
```

5. **Build:**

```bash
npm run build:wasm  # Generates .stories.js files + WASM
npm run storybook   # Start Storybook dev server
```