# Storybook Integration for storybook-rs

This directory contains Storybook configuration files that enable integration with the JavaScript Storybook API.

## Files

- **main.js**: Main Storybook configuration
- **preview.js**: Preview configuration that initializes the WASM module

## How It Works

1. The WASM module is initialized before stories are loaded
2. `register_all_stories()` registers all Rust components
3. Stories are exported in Storybook CSF (Component Story Format) compatible format
4. Storybook renders the components using the WASM bindings

## Usage with Storybook

To use with the official Storybook:

```bash
npm install --save-dev @storybook/web-components
npx storybook init
```

Then copy these configuration files to your `.storybook` directory.

## Story Format

The Rust components automatically export argTypes in Storybook-compatible format:

```json
{
  "title": "Components/Button",
  "component": "Button",
  "argTypes": {
    "label": {
      "control": { "type": "text" },
      "type": "alloc::string::String"
    },
    "color": {
      "control": { "type": "text" },
      "type": "alloc::string::String"
    }
  }
}
```