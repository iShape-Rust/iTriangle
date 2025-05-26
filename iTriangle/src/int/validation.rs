use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::IntOverlayOptions;

#[derive(Debug, Clone, Copy)]
pub struct Validation {
    pub fill_rule: FillRule,
    pub options: IntOverlayOptions,
}

impl Validation {
    pub fn with_fill_rule(fill_rule: FillRule) -> Self {
        Self {
            fill_rule,
            options: IntOverlayOptions::keep_output_points(),
        }
    }
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            fill_rule: FillRule::NonZero,
            options: IntOverlayOptions::keep_output_points(),
        }
    }
}
