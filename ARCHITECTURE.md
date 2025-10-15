# Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         Browser                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              storybook/index.html                       │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │  JavaScript Code                                  │  │ │
│  │  │  - Loads WASM module                             │  │ │
│  │  │  - Calls get_stories()                           │  │ │
│  │  │  - Calls render_story(name, args)                │  │ │
│  │  │  - Updates DOM                                    │  │ │
│  │  └──────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────┘ │
│                          ↕ WASM bindings                     │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           example/pkg/example_bg.wasm                   │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │  storybook-core (compiled to WASM)               │  │ │
│  │  │  - Story trait                                    │  │ │
│  │  │  - StoryMeta registry                            │  │ │
│  │  │  - get_stories() → JsValue                       │  │ │
│  │  │  - render_story() → Dom node                     │  │ │
│  │  └──────────────────────────────────────────────────┘  │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │  Example Components (compiled to WASM)           │  │ │
│  │  │  - Button::render()                              │  │ │
│  │  │  - Card::render()                                │  │ │
│  │  │  - Input::render()                               │  │ │
│  │  └──────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

       ↑ Compiled from Rust with wasm-pack

┌─────────────────────────────────────────────────────────────┐
│                    Rust Source Code                          │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  storybook-derive (Procedural Macro Crate)             │ │
│  │  - #[derive(Story)] macro                              │ │
│  │  - Generates story_name() and story_args()             │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  storybook-core (Core Library)                         │ │
│  │  - Story trait definition                              │ │
│  │  - StoryMeta struct                                    │ │
│  │  - Global registry with once_cell                      │ │
│  │  - WASM export functions                               │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  example (Component Implementations)                   │ │
│  │  - Component structs with #[derive(Story)]            │ │
│  │  - Story trait implementations                         │ │
│  │  - register_all_stories() function                     │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

### 1. Component Registration (at WASM initialization)

```
Browser loads example.wasm
    ↓
WASM module initialized
    ↓
example::register_all_stories() called
    ↓
For each component:
    StoryMeta { name, args, render_fn }
    ↓
    Pushed to STORY_REGISTRY (Lazy<Mutex<Vec<StoryMeta>>>)
```

### 2. Getting Story List

```
JavaScript: get_stories()
    ↓
Rust: storybook_core::get_stories()
    ↓
Lock STORY_REGISTRY
    ↓
Map each StoryMeta:
    - name (String)
    - args (Vec<ArgType>) from calling meta.args()
    ↓
Serialize to JSON
    ↓
Convert to JsValue
    ↓
Return to JavaScript
```

### 3. Rendering a Story

```
JavaScript: render_story("Button", {label: "Click", color: "#f00"})
    ↓
Rust: storybook_core::render_story(name, args)
    ↓
Find StoryMeta in STORY_REGISTRY by name
    ↓
Call (meta.render_fn)(args)
    ↓
Component's render() method:
    1. Deserialize args: JsValue → Button struct
    2. Create DOM with dominator::html! macro
    3. Return Dom node
    ↓
Get #storybook-root element from DOM
    ↓
Clear existing content
    ↓
Append new Dom node with dominator::append_dom()
    ↓
Browser updates display
```

## Key Components

### Story Trait

```rust
pub trait Story: 'static + Sync {
    fn name() -> &'static str;
    fn args() -> Vec<ArgType>;
    fn render(args: JsValue) -> Dom;
}
```

- **name()**: Returns the component identifier
- **args()**: Returns metadata about component properties
- **render()**: Creates the DOM representation

### StoryMeta Struct

```rust
pub struct StoryMeta {
    pub name: &'static str,
    pub args: fn() -> Vec<ArgType>,
    pub render_fn: fn(JsValue) -> Dom,
}
```

Stores function pointers to avoid storing trait objects (which aren't `Sync`).

### Derive Macro

```rust
#[proc_macro_derive(Story)]
pub fn derive_story(input: TokenStream) -> TokenStream {
    // Parse struct with syn
    // Extract field names and types
    // Generate helper methods with quote
    // story_name() → struct name as &'static str
    // story_args() → Vec<ArgType> with field metadata
}
```

### Global Registry

```rust
static STORY_REGISTRY: Lazy<Mutex<Vec<StoryMeta>>> = 
    Lazy::new(|| Mutex::new(Vec::new()));
```

- **Lazy**: Initialized on first access
- **Mutex**: Thread-safe access (though WASM is single-threaded)
- **Vec**: Stores all registered components

## Why This Approach?

### once_cell instead of inventory/linkme

- ✅ Works reliably with WASM targets
- ✅ No linker tricks needed
- ✅ Simple and explicit registration

### Function pointers instead of trait objects

- ✅ `fn()` types are `Sync` by default
- ✅ Avoids `Box<dyn Trait>` which requires `unsafe impl Sync`
- ✅ Cleaner for static registration

### dominator instead of raw web-sys

- ✅ Ergonomic DOM construction with html! macro
- ✅ Efficient updates with reactive signals
- ✅ Type-safe element types

### serde-wasm-bindgen for serialization

- ✅ Automatic conversion between JS objects and Rust structs
- ✅ Type-safe deserialization
- ✅ Works seamlessly with #[derive(Deserialize)]

## Build Process

```
Rust source files
    ↓
cargo build --target wasm32-unknown-unknown
    ↓
example.wasm binary
    ↓
wasm-bindgen (via wasm-pack)
    ↓
example.js (JavaScript glue)
example_bg.wasm (optimized WASM)
example.d.ts (TypeScript definitions)
    ↓
Loaded by browser via ES6 modules
```

## Performance Considerations

- **WASM size**: ~150KB unoptimized (wasm-opt disabled due to blocked domain)
- **Load time**: Near-instant on localhost
- **Render performance**: Dominated by DOM operations, not WASM
- **Memory**: Minimal - components are stateless, registry is small

## Extensibility

To add new component types:

1. Create struct with fields
2. Add `#[derive(Story, Deserialize)]`
3. Implement `Story` trait
4. Register in `register_all_stories()`
5. Rebuild with wasm-pack

No changes needed to core library or viewer!
