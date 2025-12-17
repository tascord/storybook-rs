# storybook-cli

Command-line tool for setting up [storybook-rs](https://github.com/tascord/storybook-rs) projects.

## Installation

```bash
cargo install storybook-cli
```

Or build from source:

```bash
git clone https://github.com/tascord/storybook-rs
cd storybook-rs
cargo install --path crates/storybook-cli
```

## Usage

### Initialize Storybook in Your Project

Navigate to your Rust WASM project directory and run:

```bash
storybook-cli init
```

This will:
1. Create `.storybook/main.js` with correct story patterns
2. Create `.storybook/preview.js` with WASM initialization
3. Create `vite.config.js` for Vite configuration
4. Update (or create) `package.json` with:
   - Required Storybook dependencies
   - npm scripts for building and running Storybook
5. Update `.gitignore` with Storybook-related entries

### Options

```bash
storybook-cli init [OPTIONS]

Options:
  -p, --path <PATH>
          Directory to initialize Storybook in (defaults to current directory)
          [default: .]

  -t, --target-dir <TARGET_DIR>
          Target directory for WASM build output (relative to project root)
          [default: pkg]

  -P, --port <PORT>
          Custom port for Storybook dev server
          [default: 6006]

  -s, --stories <STORIES>
          Pattern for story files (relative to target directory)
          [default: **/*.stories.@(js|jsx|ts|tsx)]

  -h, --help
          Print help information
```

### Examples

**Basic initialization:**
```bash
storybook-cli init
```

**Initialize with custom target directory:**
```bash
storybook-cli init --target-dir wasm-output
```

**Initialize for a monorepo package:**
```bash
cd packages/my-component
storybook-cli init --target-dir ../../dist/my-component
```

**Custom port:**
```bash
storybook-cli init --port 9009
```

## Monorepo Support

The CLI automatically detects your crate name from `Cargo.toml` and sets up the correct import paths. This makes it work seamlessly in monorepos where multiple packages might be building WASM modules.

## After Initialization

Once the CLI has set up your project:

1. Install npm dependencies:
   ```bash
   npm install
   ```

2. Start Storybook:
   ```bash
   npm run storybook
   ```

3. Build for production:
   ```bash
   npm run build-storybook
   ```

## What Gets Created

### `.storybook/main.js`
Storybook configuration that points to your generated story files.

### `.storybook/preview.js`
Initializes your WASM module before stories load and sets up rendering parameters.

### `vite.config.js`
Configures Vite to properly handle WASM modules.

### `package.json` updates
- Adds required `@storybook/*` dependencies
- Adds `build:wasm`, `storybook`, and `build-storybook` scripts

## Troubleshooting

### "Module not found" errors

Make sure:
1. Your crate name matches what's in `Cargo.toml`
2. You've run `npm run build:wasm` at least once
3. The `--target-dir` matches your wasm-pack output directory

### Monorepo path issues

Use absolute paths or paths relative to your project root when specifying `--target-dir` in a monorepo.

## License

MIT
