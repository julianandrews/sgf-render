use svg::node::element;

use super::StoneColor;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GobanStyle {
    line_color: String,
    line_width: f64,
    hoshi_radius: f64,
    background_fill: String,
    label_color: String,
    black_stone_fill: Option<String>,
    white_stone_fill: Option<String>,
    black_stone_stroke: Option<String>,
    white_stone_stroke: Option<String>,
    markup_stroke_width: f64,
    black_stone_markup_color: String,
    white_stone_markup_color: String,
    empty_markup_color: String,
    black_stone_selected_color: String,
    white_stone_selected_color: String,
    empty_selected_color: String,
    defs: Option<String>,
}

impl GobanStyle {
    pub fn line_color(&self) -> &str {
        &self.line_color
    }

    pub fn line_width(&self) -> f64 {
        self.line_width
    }

    pub fn hoshi_radius(&self) -> f64 {
        self.hoshi_radius
    }

    pub fn background_fill(&self) -> &str {
        &self.background_fill
    }

    pub fn label_color(&self) -> &str {
        &self.label_color
    }

    pub fn stone_fill(&self, color: StoneColor) -> Option<&str> {
        match color {
            StoneColor::Black => self.black_stone_fill.as_deref(),
            StoneColor::White => self.white_stone_fill.as_deref(),
        }
    }

    pub fn stone_stroke(&self, color: StoneColor) -> Option<&str> {
        match color {
            StoneColor::Black => self.black_stone_stroke.as_deref(),
            StoneColor::White => self.white_stone_stroke.as_deref(),
        }
    }

    pub fn markup_color(&self, color: Option<StoneColor>) -> &str {
        match color {
            Some(StoneColor::Black) => &self.black_stone_markup_color,
            Some(StoneColor::White) => &self.white_stone_markup_color,
            None => &self.empty_markup_color,
        }
    }

    pub fn markup_stroke_width(&self) -> f64 {
        self.markup_stroke_width
    }

    pub fn selected_color(&self, color: Option<StoneColor>) -> &str {
        match color {
            Some(StoneColor::Black) => &self.black_stone_selected_color,
            Some(StoneColor::White) => &self.white_stone_selected_color,
            None => &self.empty_selected_color,
        }
    }

    pub fn add_defs(&self, mut defs: element::Definitions) -> element::Definitions {
        let arrowhead = element::Marker::new()
            .set("id", "arrowhead")
            .set("markerWidth", 7)
            .set("markerHeight", 5)
            .set("refX", 7)
            .set("refY", 2.5)
            .set("orient", "auto")
            .add(element::Polygon::new().set("points", "0 0, 7 2.5, 0 5"));
        let linehead = element::Marker::new()
            .set("id", "linehead")
            .set("markerWidth", 4)
            .set("markerHeight", 4)
            .set("refX", 2)
            .set("refY", 2)
            .add(element::Circle::new().set("cx", 2).set("cy", 2).set("r", 2));
        defs = defs.add(linehead).add(arrowhead);
        if let Some(_s) = &self.defs {
            todo!();
        }
        defs
    }
}
