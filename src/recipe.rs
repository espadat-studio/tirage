use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::{DERIVATION_MAJOR, Error, Palette, SonarParams, sonar};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Sonar,
}

impl Tool {
    pub const ALL: &[Tool] = &[Tool::Sonar];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Sonar => sonar::SLUG,
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
        1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Params {
    Sonar(SonarParams),
}

impl Params {
    pub fn tool(&self) -> Tool {
        match self {
            Self::Sonar(_) => Tool::Sonar,
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
