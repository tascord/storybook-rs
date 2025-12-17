use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "storybook-cli")]
#[command(about = "CLI tool for setting up storybook-rs projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Storybook configuration in your project
    Init {
        /// Directory to initialize Storybook in (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Target directory for WASM build output (relative to project root)
        #[arg(short, long, default_value = "pkg")]
        target_dir: String,

        /// Custom port for Storybook dev server
        #[arg(short = 'P', long, default_value = "6006")]
        port: u16,

        /// Pattern for story files (relative to target directory)
        #[arg(short, long, default_value = "**/*.stories.@(js|jsx|ts|tsx)")]
        stories: String,
    },
}

const MAIN_JS_TEMPLATE: &str = r#"// Storybook configuration for Rust WASM components
// This integrates the Rust-generated stories with Storybook's JS API

export default {
  stories: [
    '{TARGET_DIR}/{STORIES_PATTERN}'
  ],

  addons: ['@storybook/addon-links', '@storybook/addon-docs'],

  framework: {
    name: '@storybook/web-components-vite',
    options: {},
  }
};
"#;

const PREVIEW_JS_TEMPLATE: &str = r#"// Preview configuration for Rust WASM components
// This file initializes the WASM module and sets up the rendering environment

import init, { register_all_stories } from '{WASM_IMPORT_PATH}';

// Initialize WASM module before stories load
let wasmInitialized = false;

export async function loadGlobalSetup() {
  if (!wasmInitialized) {
    await init();
    register_all_stories();
    wasmInitialized = true;
  }
}

// Call setup before rendering
loadGlobalSetup();

export const parameters = {
  actions: { argTypesRegex: '^on[A-Z].*' },
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
};
"#;

const VITE_CONFIG_TEMPLATE: &str = r#"import { defineConfig } from 'vite';

export default defineConfig({
  optimizeDeps: {
    exclude: ['{TARGET_DIR}'],
  },
  server: {
    fs: {
      // Allow serving files from the target directory
      allow: ['..'],
    },
  },
});
"#;

fn init_storybook(
    path: PathBuf,
    target_dir: String,
    port: u16,
    stories_pattern: String,
) -> Result<()> {
    let project_dir = path.canonicalize()
        .context("Failed to resolve project directory")?;
    
    println!("🚀 Initializing Storybook in: {}", project_dir.display());

    // Create .storybook directory
    let storybook_dir = project_dir.join(".storybook");
    fs::create_dir_all(&storybook_dir)
        .context("Failed to create .storybook directory")?;
    println!("✓ Created .storybook directory");

    // Find the Cargo.toml to determine the crate name
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let crate_name = if cargo_toml_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml_path)
            .context("Failed to read Cargo.toml")?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_content)
            .context("Failed to parse Cargo.toml")?;
        
        cargo_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("example")
            .replace("-", "_") // Rust converts hyphens to underscores in crate names
    } else {
        println!("⚠ Warning: Cargo.toml not found, using 'example' as crate name");
        "example".to_string()
    };

    // Create main.js with correct paths
    let main_js_content = MAIN_JS_TEMPLATE
        .replace("{TARGET_DIR}", &target_dir)
        .replace("{STORIES_PATTERN}", &stories_pattern);
    fs::write(storybook_dir.join("main.js"), main_js_content)
        .context("Failed to write main.js")?;
    println!("✓ Created .storybook/main.js");

    // Create preview.js with correct import path
    let wasm_import_path = format!("../{}/{}.js", target_dir, crate_name);
    let preview_js_content = PREVIEW_JS_TEMPLATE
        .replace("{WASM_IMPORT_PATH}", &wasm_import_path);
    fs::write(storybook_dir.join("preview.js"), preview_js_content)
        .context("Failed to write preview.js")?;
    println!("✓ Created .storybook/preview.js");

    // Create or update vite.config.js
    let vite_config_path = project_dir.join("vite.config.js");
    if !vite_config_path.exists() {
        let vite_config_content = VITE_CONFIG_TEMPLATE
            .replace("{TARGET_DIR}", &target_dir);
        fs::write(&vite_config_path, vite_config_content)
            .context("Failed to write vite.config.js")?;
        println!("✓ Created vite.config.js");
    } else {
        println!("⚠ vite.config.js already exists, skipping");
    }

    // Create or update package.json
    let package_json_path = project_dir.join("package.json");
    update_package_json(&package_json_path, &target_dir, port)?;

    // Add .storybook to .gitignore if it's not already there
    update_gitignore(&project_dir)?;

    println!("\n✅ Storybook initialization complete!");
    println!("\nNext steps:");
    println!("  1. Install dependencies: npm install");
    println!("  2. Build your WASM: npm run build:wasm");
    println!("  3. Start Storybook: npm run storybook");
    println!("\nYour components should implement the Story trait and be registered with:");
    println!("  storybook::register_stories!(YourComponent1, YourComponent2);");

    Ok(())
}

fn update_package_json(path: &Path, target_dir: &str, port: u16) -> Result<()> {
    let mut package_json: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(path)
            .context("Failed to read package.json")?;
        serde_json::from_str(&content)
            .context("Failed to parse package.json")?
    } else {
        serde_json::json!({
            "name": "storybook-project",
            "version": "1.0.0",
            "private": true
        })
    };

    // Add or update scripts
    let scripts = package_json
        .get_mut("scripts")
        .and_then(|s| s.as_object_mut())
        .map(|s| s.clone())
        .unwrap_or_default();

    let mut new_scripts = serde_json::Map::new();
    for (key, value) in scripts {
        new_scripts.insert(key, value);
    }

    new_scripts.insert(
        "build:wasm".to_string(),
        serde_json::Value::String(format!("wasm-pack build --target web --out-dir {}", target_dir)),
    );
    new_scripts.insert(
        "storybook".to_string(),
        serde_json::Value::String(format!("npm run build:wasm && storybook dev -p {}", port)),
    );
    new_scripts.insert(
        "build-storybook".to_string(),
        serde_json::Value::String("npm run build:wasm && storybook build".to_string()),
    );

    package_json["scripts"] = serde_json::Value::Object(new_scripts);

    // Add dev dependencies if not present
    let dev_deps = package_json
        .get_mut("devDependencies")
        .and_then(|d| d.as_object_mut())
        .map(|d| d.clone())
        .unwrap_or_default();

    let mut new_dev_deps = serde_json::Map::new();
    for (key, value) in dev_deps {
        new_dev_deps.insert(key, value);
    }

    // Add required dependencies if they don't exist
    let required_deps = vec![
        ("@storybook/addon-links", "^10.0.7"),
        ("@storybook/addon-docs", "^10.0.7"),
        ("@storybook/web-components-vite", "^10.0.7"),
        ("storybook", "^10.0.7"),
        ("vite", "^5.0.0"),
    ];

    for (dep, version) in required_deps {
        if !new_dev_deps.contains_key(dep) {
            new_dev_deps.insert(dep.to_string(), serde_json::Value::String(version.to_string()));
        }
    }

    package_json["devDependencies"] = serde_json::Value::Object(new_dev_deps);

    // Write back to file with pretty formatting
    let content = serde_json::to_string_pretty(&package_json)
        .context("Failed to serialize package.json")?;
    fs::write(path, content)
        .context("Failed to write package.json")?;
    
    println!("✓ Updated package.json with scripts and dependencies");
    Ok(())
}

fn update_gitignore(project_dir: &Path) -> Result<()> {
    let gitignore_path = project_dir.join(".gitignore");
    let entries_to_add = vec![
        "node_modules",
        "package-lock.json",
        "storybook-static",
        ".storybook",
    ];

    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)
            .context("Failed to read .gitignore")?
    } else {
        String::new()
    };

    let mut new_entries = Vec::new();
    for entry in entries_to_add {
        if !existing_content.contains(entry) {
            new_entries.push(entry);
        }
    }

    if !new_entries.is_empty() {
        let mut updated_content = existing_content;
        if !updated_content.is_empty() && !updated_content.ends_with('\n') {
            updated_content.push('\n');
        }
        updated_content.push_str("\n# Storybook\n");
        for entry in new_entries {
            updated_content.push_str(entry);
            updated_content.push('\n');
        }

        fs::write(&gitignore_path, updated_content)
            .context("Failed to write .gitignore")?;
        println!("✓ Updated .gitignore");
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            path,
            target_dir,
            port,
            stories,
        } => {
            init_storybook(path, target_dir, port, stories)?;
        }
    }

    Ok(())
}
