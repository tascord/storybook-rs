use dominator::{html, Dom};
use serde::Deserialize;
use storybook::Story;
use storybook::{StoryDerive, StorySelect};

/// Button size variants
#[derive(StorySelect, Deserialize, Clone, Debug)]
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
#[derive(StoryDerive, Deserialize)]
pub struct Button {
    #[story(default = "'Click Me!'")]
    pub label: String,
    #[story(control = "color", default = "'#007bff'")]
    pub color: String,
    pub disabled: Option<bool>,
}

impl Story for Button {
    /// Convert this button into a Dom node using dominator's builder pattern
    fn into_dom(self) -> Dom {
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

/// A simple card component with auto-registration
#[derive(StoryDerive, Deserialize)]
pub struct Card {
    pub title: String,
    pub content: String,
    #[story(control = "color")]
    pub background: String,
}

impl Story for Card {
    /// Convert this card into a Dom node using dominator's builder pattern
    fn into_dom(self) -> Dom {
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

/// A simple text input component with auto-registration
#[derive(StoryDerive, Deserialize)]
pub struct Input {
    pub placeholder: String,
    pub value: String,
}

impl Story for Input {
    /// Convert this input into a Dom node using dominator's builder pattern
    fn into_dom(self) -> Dom {
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

/// Alert severity levels
#[derive(StorySelect, Deserialize, Clone, Debug)]
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
#[derive(StoryDerive, Deserialize)]
pub struct Alert {
    pub message: String,
    #[story(control = "select")]
    pub alert_type: AlertType,
}

impl Story for Alert {
    fn into_dom(self) -> Dom {
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

// Automatically generate registration function using macro
storybook::register_stories!(Button, Card, Input, Alert);
storybook::register_enums!(AlertType, ButtonSize);
