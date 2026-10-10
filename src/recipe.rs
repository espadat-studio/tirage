use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::unique_keys;
use crate::{DERIVATION_MAJOR, Error, Palette, Params, Tool};

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

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    tool_seed: NonZeroU32,
    palette: Option<Palette>,
    params: Params,
}

#[derive(Serialize)]
struct WireOut<'a> {
    tirage: u32,
    tool: &'static str,
    tool_seed: NonZeroU32,
    #[serde(skip_serializing_if = "Option::is_none")]
    palette: Option<Vec<String>>,
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
    palette: Option<Vec<String>>,
    #[serde(deserialize_with = "unique_params")]
    params: serde_json::Value,
}

fn unique_params<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<serde_json::Value, D::Error> {
    let entries = unique_keys::deserialize(deserializer, "params: ")?;
    Ok(serde_json::Value::Object(entries.into_iter().collect()))
}

impl Recipe {
    pub fn new(
        tool_seed: NonZeroU32,
        palette: Option<Palette>,
        params: Params,
    ) -> Result<Self, Error> {
        let palette = fit(params.tool(), palette)?;
        Ok(Self {
            tool_seed,
            palette,
            params,
        })
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

    pub fn palette(&self) -> Option<&Palette> {
        self.palette.as_ref()
    }

    pub fn set_palette(&mut self, palette: Palette) -> Result<(), Error> {
        self.palette = fit(self.tool(), Some(palette))?;
        Ok(())
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
            palette: self.palette.as_ref().map(Palette::to_hex),
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
        let params = Tool::from_slug(&wire.tool)?.params_from_json(wire.params)?;
        let palette = wire.palette.as_deref().map(Palette::from_hex).transpose()?;
        Self::new(wire.tool_seed, palette, params)
    }
}

fn fit(tool: Tool, palette: Option<Palette>) -> Result<Option<Palette>, Error> {
    let palette = match (tool.palette(), palette) {
        (None, None) => return Ok(None),
        (None, Some(_)) => return Err(Error::NoPalette(tool)),
        (Some(_), None) => return Err(Error::FewInks(0)),
        (Some(_), Some(palette)) => palette,
    };
    match tool.max_inks() {
        Some(max) if palette.len() > max => Err(Error::TooManyInks {
            tool,
            inks: palette.len(),
            max,
        }),
        _ => Ok(Some(palette)),
    }
}

fn json_error(error: serde_json::Error) -> Error {
    Error::Json(error.to_string())
}
