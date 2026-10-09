use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::param::Param;
use crate::{
    DERIVATION_MAJOR, Error, HuskParams, Palette, SonarParams, VeinParams, husk, sonar, vein,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Sonar,
    Husk,
    Vein,
}

impl Tool {
    pub const ALL: &[Tool] = &[Tool::Sonar, Tool::Husk, Tool::Vein];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Sonar => sonar::SLUG,
            Self::Husk => husk::SLUG,
            Self::Vein => vein::SLUG,
        }
    }

    pub fn from_slug(slug: &str) -> Result<Self, Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|tool| tool.slug() == slug)
            .ok_or_else(|| Error::UnknownTool(slug.to_owned()))
    }

    pub fn frames(self) -> u32 {
        match self {
            Self::Sonar => sonar::FRAMES,
            Self::Husk => husk::FRAMES,
            Self::Vein => vein::FRAMES,
        }
    }

    pub fn fps(self) -> u32 {
        match self {
            Self::Sonar => sonar::FPS,
            Self::Husk => husk::FPS,
            Self::Vein => vein::FPS,
        }
    }

    pub fn parameters(self) -> Vec<Parameter> {
        match self {
            Self::Sonar => sonar::parameters(),
            Self::Husk => husk::parameters(),
            Self::Vein => vein::parameters(),
        }
    }

    pub(crate) fn params(self) -> &'static [Param] {
        match self {
            Self::Sonar => sonar::PARAMS,
            Self::Husk => husk::PARAMS,
            Self::Vein => vein::PARAMS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Parameter {
    pub id: &'static str,
    #[serde(flatten)]
    pub kind: ParameterKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ParameterKind {
    Range { min: f64, max: f64, step: f64 },
    Toggle,
    Choice { choices: &'static [&'static str] },
}

impl Parameter {
    pub(crate) fn toggle(id: &'static str) -> Self {
        Self {
            id,
            kind: ParameterKind::Toggle,
        }
    }

    pub(crate) fn choice(id: &'static str, choices: &'static [&'static str]) -> Self {
        Self {
            id,
            kind: ParameterKind::Choice { choices },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Params {
    Sonar(SonarParams),
    Husk(HuskParams),
    Vein(VeinParams),
}

impl Params {
    pub fn tool(&self) -> Tool {
        match self {
            Self::Sonar(_) => Tool::Sonar,
            Self::Husk(_) => Tool::Husk,
            Self::Vein(_) => Tool::Vein,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    tool_seed: NonZeroU32,
    palette: Palette,
    params: Params,
}

#[derive(Serialize)]
struct WireOut<'a> {
    tirage: u32,
    tool: &'static str,
    tool_seed: NonZeroU32,
    palette: Vec<String>,
    params: &'a Params,
}

#[derive(Deserialize)]
struct WireMajor {
    tirage: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireIn {
    #[serde(rename = "tirage")]
    _tirage: u64,
    tool: String,
    tool_seed: NonZeroU32,
    palette: Vec<String>,
    params: serde_json::Value,
}

impl Recipe {
    pub fn new(tool_seed: NonZeroU32, palette: Palette, params: Params) -> Self {
        Self {
            tool_seed,
            palette,
            params,
        }
    }

    pub fn tool(&self) -> Tool {
        self.params.tool()
    }

    pub fn tool_seed(&self) -> NonZeroU32 {
        self.tool_seed
    }

    pub fn set_tool_seed(&mut self, tool_seed: NonZeroU32) {
        self.tool_seed = tool_seed;
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut Params {
        &mut self.params
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&WireOut {
            tirage: DERIVATION_MAJOR,
            tool: self.tool().slug(),
            tool_seed: self.tool_seed,
            palette: self.palette.to_hex(),
            params: &self.params,
        })
        .expect("a Recipe always serializes")
    }

    pub fn from_json(json: &str) -> Result<Self, Error> {
        let WireMajor { tirage } = serde_json::from_str(json).map_err(json_error)?;
        if tirage != u64::from(DERIVATION_MAJOR) {
            return Err(Error::Major(tirage));
        }
        let wire: WireIn = serde_json::from_str(json).map_err(json_error)?;
        let params = match Tool::from_slug(&wire.tool)? {
            Tool::Sonar => Params::Sonar(sonar::from_json(wire.params)?),
            Tool::Husk => Params::Husk(husk::from_json(wire.params)?),
            Tool::Vein => Params::Vein(vein::from_json(wire.params)?),
        };
        Ok(Self {
            tool_seed: wire.tool_seed,
            palette: Palette::from_hex(&wire.palette)?,
            params,
        })
    }
}

fn json_error(error: serde_json::Error) -> Error {
    Error::Json(error.to_string())
}
