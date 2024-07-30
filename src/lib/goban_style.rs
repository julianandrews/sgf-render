use minidom::Element;

use crate::errors::MakeSvgError;
use crate::generated_styles::GeneratedStyle;
use crate::goban::StoneColor;
use crate::make_svg::NAMESPACE;

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

    pub fn defs(&self) -> Result<Vec<Element>, MakeSvgError> {
        let linehead = Element::builder("marker", NAMESPACE)
            .attr("id", "linehead")
            .attr("markerWidth", "4")
            .attr("markerHeight", "4")
            .attr("refX", "2")
            .attr("refY", "2")
            .append(
                Element::builder("circle", NAMESPACE)
                    .attr("cx", "2")
                    .attr("cy", "2")
                    .attr("r", "2")
                    .build(),
            )
            .build();
        let arrowhead = Element::builder("marker", NAMESPACE)
            .attr("id", "arrowhead")
            .attr("markerWidth", "7")
            .attr("markerHeight", "5")
            .attr("refX", "7")
            .attr("refY", "2.5")
            .attr("orient", "auto")
            .append(
                Element::builder("polygon", NAMESPACE)
                    .attr("points", "0 0, 7 2.5, 0 5")
                    .build(),
            )
            .build();
        let mut defs = vec![linehead, arrowhead];
        if let Some(s) = &self.defs {
            // Wrap
            let wrapped = format!("<svg xmlns=\"{}\">{}</svg>", NAMESPACE, s);
            let wrapper: Element = wrapped.parse().map_err(MakeSvgError::StyleDefError)?;
            for child in wrapper.children() {
                defs.push(child.clone());
            }
        }
        Ok(defs)
    }
}

impl Default for GobanStyle {
    fn default() -> Self {
        GeneratedStyle::Simple.style().clone()
    }
}
