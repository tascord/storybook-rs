use storybook_core::Story;
use storybook_derive::{Story as DeriveStory, StorySelect as DeriveStorySelect};
use dominator::{Dom, html};
use wasm_bindgen::prelude::*;
use serde::Deserialize;

/// Button size variants
#[derive(DeriveStorySelect, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl Default for ButtonSize {
    fn default() -> Self {
        ButtonSize::Medium
    }
}

impl ButtonSize {
    #[allow(dead_code)]
    fn to_css(&self) -> &'static str {
        match self {
            ButtonSize::Small => "8px 16px",
            ButtonSize::Medium => "10px 20px",
            ButtonSize::Large => "12px 24px",
        }
    }
}

/// A simple button component with auto-registration
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
        let button: Button = serde_wasm_bindgen::from_value(args).unwrap_or(Button {
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
            .style("font-size", "16px")
        })
    }
}

/// A simple card component with auto-registration
#[derive(DeriveStory, Deserialize)]
pub struct Card {
    pub title: String,
    pub content: String,
    pub background: String,
}

impl Story for Card {
    fn name() -> &'static str {
        Card::story_name()
    }

    fn args() -> Vec<storybook_core::ArgType> {
        Card::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let card: Card = serde_wasm_bindgen::from_value(args).unwrap_or(Card {
            title: "Card Title".to_string(),
            content: "This is card content".to_string(),
            background: "#ffffff".to_string(),
        });
        
        html!("div", {
            .style("background-color", &card.background)
            .style("border", "1px solid #ddd")
            .style("border-radius", "8px")
            .style("padding", "20px")
            .style("box-shadow", "0 2px 4px rgba(0,0,0,0.1)")
            .style("max-width", "400px")
            .children(&mut [
                html!("h2", {
                    .text(&card.title)
                    .style("margin-top", "0")
                    .style("margin-bottom", "10px")
                }),
                html!("p", {
                    .text(&card.content)
                    .style("margin", "0")
                    .style("color", "#666")
                }),
            ])
        })
    }
}

/// A simple text input component with auto-registration
#[derive(DeriveStory, Deserialize)]
pub struct Input {
    pub placeholder: String,
    pub value: String,
}

impl Story for Input {
    fn name() -> &'static str {
        Input::story_name()
    }

    fn args() -> Vec<storybook_core::ArgType> {
        Input::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let input: Input = serde_wasm_bindgen::from_value(args).unwrap_or(Input {
            placeholder: "Enter text...".to_string(),
            value: "".to_string(),
        });
        
        html!("input" => web_sys::HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", &input.placeholder)
            .attr("value", &input.value)
            .style("padding", "10px")
            .style("border", "1px solid #ccc")
            .style("border-radius", "4px")
            .style("font-size", "14px")
            .style("width", "200px")
        })
    }
}

// Automatically generate registration function using macro
storybook_core::register_stories!(Button, Card, Input);
