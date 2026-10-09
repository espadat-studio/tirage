use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::param::Param;
use crate::unique_keys;
use crate::{
    AuraParams, DERIVATION_MAJOR, Error, FrondParams, HuskParams, KioskParams, Palette,
    SonarParams, VeinParams, aura, frond, husk, kiosk, sonar, vein,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Sonar,
    Husk,
    Vein,
    Aura,
    Kiosk,
    Frond,
}

impl Tool {
    pub const ALL: &[Tool] = &[
        Tool::Sonar,
        Tool::Husk,
        Tool::Vein,
        Tool::Aura,
        Tool::Kiosk,
        Tool::Frond,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Sonar => sonar::SLUG,
            Self::Husk => husk::SLUG,
            Self::Vein => vein::SLUG,
            Self::Aura => aura::SLUG,
            Self::Kiosk => kiosk::SLUG,
            Self::Frond => frond::SLUG,
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
            Self::Aura => aura::FRAMES,
            Self::Kiosk => kiosk::FRAMES,
            Self::Frond => frond::FRAMES,
        }
    }

    pub fn fps(self) -> u32 {
        match self {
            Self::Sonar => sonar::FPS,
            Self::Husk => husk::FPS,
            Self::Vein => vein::FPS,
            Self::Aura => aura::FPS,
            Self::Kiosk => kiosk::FPS,
            Self::Frond => frond::FPS,
        }
    }

    pub fn parameters(self) -> Vec<Parameter> {
        match self {
            Self::Sonar => sonar::parameters(),
            Self::Husk => husk::parameters(),
            Self::Vein => vein::parameters(),
            Self::Aura => aura::parameters(),
            Self::Kiosk => kiosk::parameters(),
            Self::Frond => frond::parameters(),
        }
    }

    pub(crate) fn params(self) -> &'static [Param] {
        match self {
            Self::Sonar => sonar::PARAMS,
            Self::Husk => husk::PARAMS,
            Self::Vein => vein::PARAMS,
            Self::Aura => aura::PARAMS,
            Self::Kiosk => kiosk::PARAMS,
            Self::Frond => frond::PARAMS,
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
    Aura(AuraParams),
    Kiosk(KioskParams),
    Frond(FrondParams),
}

impl Params {
    pub fn tool(&self) -> Tool {
        match self {
            Self::Sonar(_) => Tool::Sonar,
            Self::Husk(_) => Tool::Husk,
            Self::Vein(_) => Tool::Vein,
            Self::Aura(_) => Tool::Aura,
            Self::Kiosk(_) => Tool::Kiosk,
            Self::Frond(_) => Tool::Frond,
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
    pub fn new(tool_seed: NonZeroU32, palette: Palette, params: Params) -> Result<Self, Error> {
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

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn set_palette(&mut self, palette: Palette) -> Result<(), Error> {
        self.palette = fit(self.tool(), palette)?;
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
            Tool::Aura => Params::Aura(aura::from_json(wire.params)?),
            Tool::Kiosk => Params::Kiosk(kiosk::from_json(wire.params)?),
            Tool::Frond => Params::Frond(frond::from_json(wire.params)?),
        };
        Self::new(wire.tool_seed, Palette::from_hex(&wire.palette)?, params)
    }
}

fn fit(tool: Tool, palette: Palette) -> Result<Palette, Error> {
    let max = match tool {
        Tool::Aura => aura::INKS,
        Tool::Sonar | Tool::Husk | Tool::Vein | Tool::Kiosk | Tool::Frond => return Ok(palette),
    };
    if palette.len() > max {
        return Err(Error::TooManyInks {
            tool,
            inks: palette.len(),
            max,
        });
    }
    Ok(palette)
}

fn json_error(error: serde_json::Error) -> Error {
    Error::Json(error.to_string())
}
