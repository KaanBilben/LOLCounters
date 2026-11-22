//! Lane tanımı (havuz kaydı LC-003'te eklenir).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lane {
    Top,
    Jungle,
    Mid,
    Adc,
    Support,
}

impl Lane {
    pub const ALL: [Lane; 5] = [Lane::Top, Lane::Jungle, Lane::Mid, Lane::Adc, Lane::Support];
}
