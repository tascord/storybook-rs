use storybook::Story;
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
    #[story(default = "'Click Me!'")]
    pub label: String,
    #[story(control = "color", default = "'#007bff'")]
    pub color: String,
    pub disabled: Option<bool>,
}

impl Button {
    /// Convert this button into a Dom node using dominator's builder pattern
    pub fn into_dom(self) -> Dom {
        let is_disabled = self.disabled.unwrap_or(false);
        html!("button", {
            .text(&self.label)
            .style("background-color", &self.color)
            .style("color", "white")
            .style("border", "none")
            .style("padding", "10px 20px")
            .style("border-radius", "4px")
            .style("cursor", if is_disabled { "not-allowed" } else { "pointer" })
            .style("font-size", "16px")
            .style("opacity", if is_disabled { "0.5" } else { "1" })
            .prop("disabled", is_disabled)
        })
    }
}

impl Story for Button {
    fn name() -> &'static str {
        Button::story_name()
    }

    fn args() -> Vec<storybook::ArgType> {
        Button::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let button: Button = serde_wasm_bindgen::from_value(args).unwrap_or(Button {
            label: "Click me".to_string(),
            color: "#007bff".to_string(),
            disabled: None,
        });
        
        // Use the into_dom method
        button.into_dom()
    }
}

/// A simple card component with auto-registration
#[derive(DeriveStory, Deserialize)]
pub struct Card {
    pub title: String,
    pub content: String,
    #[story(control = "color")]
    pub background: String,
}

impl Card {
    /// Convert this card into a Dom node using dominator's builder pattern
    pub fn into_dom(self) -> Dom {
        html!("div", {
            .style("background-color", &self.background)
            .style("border", "1px solid #ddd")
            .style("border-radius", "8px")
            .style("padding", "20px")
            .style("box-shadow", "0 2px 4px rgba(0,0,0,0.1)")
            .style("max-width", "400px")
            .children(&mut [
                html!("h2", {
                    .text(&self.title)
                    .style("margin-top", "0")
                    .style("margin-bottom", "10px")
                }),
                html!("p", {
                    .text(&self.content)
                    .style("margin", "0")
                    .style("color", "#666")
                }),
            ])
        })
    }
}

impl Story for Card {
    fn name() -> &'static str {
        Card::story_name()
    }

    fn args() -> Vec<storybook::ArgType> {
        Card::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let card: Card = serde_wasm_bindgen::from_value(args).unwrap_or(Card {
            title: "Card Title".to_string(),
            content: "This is card content".to_string(),
            background: "#ffffff".to_string(),
        });
        
        // Use the into_dom method
        card.into_dom()
    }
}

/// A simple text input component with auto-registration
#[derive(DeriveStory, Deserialize)]
pub struct Input {
    pub placeholder: String,
    pub value: String,
}

impl Input {
    /// Convert this input into a Dom node using dominator's builder pattern
    pub fn into_dom(self) -> Dom {
        html!("input" => web_sys::HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", &self.placeholder)
            .attr("value", &self.value)
            .style("padding", "10px")
            .style("border", "1px solid #ccc")
            .style("border-radius", "4px")
            .style("font-size", "14px")
            .style("width", "200px")
        })
    }
}

impl Story for Input {
    fn name() -> &'static str {
        Input::story_name()
    }

    fn args() -> Vec<storybook::ArgType> {
        Input::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let input: Input = serde_wasm_bindgen::from_value(args).unwrap_or(Input {
            placeholder: "Enter text...".to_string(),
            value: "".to_string(),
        });
        
        // Use the into_dom method
        input.into_dom()
    }
}

/// Alert severity levels
#[derive(DeriveStorySelect, Deserialize, Clone, Debug)]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for AlertType {
    fn default() -> Self {
        AlertType::Info
    }
}

impl AlertType {
    fn to_color(&self) -> &'static str {
        match self {
            AlertType::Info => "#3498db",
            AlertType::Success => "#2ecc71",
            AlertType::Warning => "#f39c12",
            AlertType::Error => "#e74c3c",
        }
    }
}

/// An alert component demonstrating enum select controls
#[derive(DeriveStory, Deserialize)]
pub struct Alert {
    pub message: String,
    #[story(control = "select")]
    pub alert_type: AlertType,
}

impl Alert {
    pub fn into_dom(self) -> Dom {
        html!("div", {
            .text(&self.message)
            .style("padding", "15px 20px")
            .style("border-radius", "4px")
            .style("background-color", self.alert_type.to_color())
            .style("color", "white")
            .style("font-weight", "500")
            .style("margin", "10px 0")
        })
    }
}

impl Story for Alert {
    fn name() -> &'static str {
        Alert::story_name()
    }

    fn args() -> Vec<storybook::ArgType> {
        Alert::story_args()
    }

    fn render(args: JsValue) -> Dom {
        let alert: Alert = serde_wasm_bindgen::from_value(args).unwrap_or(Alert {
            message: "This is an alert message".to_string(),
            alert_type: AlertType::default(),
        });
        
        alert.into_dom()
    }
}

// Automatically generate registration function using macro
storybook::register_stories!(Button, Card, Input, Alert);

// Also need to register enums - this should be called before stories are used
#[wasm_bindgen]
pub fn init_enums() {
    AlertType::__register_enum_options();
}
