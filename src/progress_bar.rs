use cosmic::iced::{Border, Color, Length, Shadow};
use cosmic::widget::{container, text};
use cosmic::Element;

#[derive(Debug, Clone)]
pub struct ProgressBar;

impl ProgressBar {
    pub fn new<'a, Message: 'static>(is_dark: bool, percent: f32) -> Element<'a, Message> {
        let background_color = if is_dark {
            Color::from_rgb(0.2, 0.2, 0.2) // #333333
        } else {
            Color::from_rgb(0.9, 0.9, 0.9) // #E6E6E6
        };

        let bar_color = if is_dark {
            Color::from_rgb(0.827, 0.827, 0.827) // #D3D3D3
        } else {
            Color::from_rgb(0.294, 0.294, 0.294) // #4B4B4B
        };

        let border_color = if is_dark {
            Color::from_rgb(0.0, 0.0, 0.0) // #D3D3D3
        } else {
            Color::from_rgb(0.294, 0.294, 0.294) // #4B4B4B
        };

        let total_width = 200.0;
        let bar_width = Length::Fixed(total_width * percent.clamp(0.0, 100.0) / 100.0);

        container(
            container(text(""))
                .width(bar_width)
                .style(move |_theme| container::Style {
                    background: Some(bar_color.into()),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 2.35,
                        radius: 2.0.into(),
                    },
                    text_color: None,
                    icon_color: None,
                    shadow: Shadow::default(),
                })
        )
            .width(Length::Fixed(total_width))
            .height(Length::Fixed(10.0)) // Slimmer bar
            .style(move |_theme| container::Style {
                background: Some(background_color.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: None,
                icon_color: None,
                shadow: Shadow::default(),
            })
            .into()
    }
}