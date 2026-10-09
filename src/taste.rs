use std::collections::BTreeMap;

use serde::Deserialize;

use crate::param::Param;
use crate::{Error, Tool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taste {
    tool: Tool,
    ticks: Vec<(u32, u32)>,
}

#[derive(Deserialize)]
struct Wire {
    tool: String,
    #[serde(flatten)]
    bounds: BTreeMap<String, [f64; 2]>,
}

impl Taste {
    pub fn new<S: AsRef<str>>(tool: Tool, bounds: &[(S, [f64; 2])]) -> Result<Self, Error> {
        let mut taste = Self::shipped(tool);
        for (id, [low, high]) in bounds {
            let id = id.as_ref();
            let slot = tool
                .params()
                .iter()
                .position(|param| param.id == id)
                .ok_or_else(|| Error::UnknownParameter {
                    tool,
                    param: id.to_owned(),
                })?;
            let param = &tool.params()[slot];
            let ticks = (param.ticks(*low)?, param.ticks(*high)?);
            if ticks.0 > ticks.1 {
                return Err(Error::Reversed {
                    scope: param.scope,
                    param: param.id,
                    low: *low,
                    high: *high,
                });
            }
            taste.ticks[slot] = ticks;
        }
        Ok(taste)
    }

    pub fn from_json(json: &str) -> Result<Self, Error> {
        let wire: Wire = serde_json::from_str(json).map_err(|e| Error::TasteJson(e.to_string()))?;
        let bounds: Vec<_> = wire.bounds.into_iter().collect();
        Self::new(Tool::from_slug(&wire.tool)?, &bounds)
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub(crate) fn shipped(tool: Tool) -> Self {
        Self {
            tool,
            ticks: tool.params().iter().map(|param| param.taste).collect(),
        }
    }

    pub(crate) fn ticks(&self, param: &Param) -> (u32, u32) {
        let slot = self
            .tool
            .params()
            .iter()
            .position(|known| known.id == param.id)
            .expect("a Tool deals only its own Parameters");
        self.ticks[slot]
    }
}
