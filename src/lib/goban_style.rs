use svg::node::element;

use super::StoneColor;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub enum GobanStyle {
    Fancy,
    Simple,
    Minimalist,
}

impl GobanStyle {
    pub fn label_color(self) -> String {
        match self {
            Self::Fancy | Self::Simple => "#6e5840".to_string(),
            Self::Minimalist => "black".to_string(),
        }
    }

    pub fn background_fill(self) -> String {
        match self {
            Self::Fancy | Self::Simple => "#cfa87e".to_string(),
            Self::Minimalist => "white".to_string(),
        }
    }

    pub fn markup_color(self, color: Option<StoneColor>) -> String {
        match color {
            None | Some(StoneColor::White) => "black".to_string(),
            Some(StoneColor::Black) => "white".to_string(),
        }
    }

    pub fn selected_color(self, color: Option<StoneColor>) -> String {
        match self {
            Self::Minimalist => match color {
                Some(StoneColor::Black) => "white".to_string(),
                _ => "black".to_string(),
            },
            _ => "blue".to_string(),
        }
    }

    pub fn arrowhead(self) -> element::Marker {
        element::Marker::new()
            .set("markerWidth", 7)
            .set("markerHeight", 5)
            .set("refX", 7)
            .set("refY", 2.5)
            .set("orient", "auto")
            .add(element::Polygon::new().set("points", "0 0, 7 2.5, 0 5"))
    }

    pub fn linehead(self) -> element::Marker {
        element::Marker::new()
            .set("markerWidth", 4)
            .set("markerHeight", 4)
            .set("refX", 2)
            .set("refY", 2)
            .add(element::Circle::new().set("cx", 2).set("cy", 2).set("r", 2))
    }

    pub fn defs(self) -> Vec<impl svg::node::Node> {
        match self {
            Self::Fancy => {
                let black_stone_fill = element::RadialGradient::new()
                    .set("id", "black-stone-fill")
                    .set("cx", "35%")
                    .set("cy", "35%")
                    .add(
                        element::Stop::new()
                            .set("offset", "0%")
                            .set("stop-color", "#666"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "100%")
                            .set("stop-color", "black"),
                    );
                let white_stone_fill = element::RadialGradient::new()
                    .set("id", "white-stone-fill")
                    .set("cx", "35%")
                    .set("cy", "35%")
                    .add(
                        element::Stop::new()
                            .set("offset", "0%")
                            .set("stop-color", "#eee"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "30%")
                            .set("stop-color", "#ddd"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "100%")
                            .set("stop-color", "#bbb"),
                    );
                vec![black_stone_fill, white_stone_fill]
            }
            Self::Simple | Self::Minimalist => vec![],
        }
    }
}
