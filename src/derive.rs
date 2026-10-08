use std::num::NonZeroU32;

use crate::{Params, Recipe, Tool, sonar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPin {
    Tool(Tool),
}

pub fn derive(seed: u64, pin: ToolPin) -> Recipe {
    let ToolPin::Tool(tool) = pin;
    let draw = |param: &str| keyed_hash(seed, &[tool.slug(), param]);
    let (palette, params) = match tool {
        Tool::Sonar => (sonar::palette(), Params::Sonar(sonar::deal(draw))),
    };
    let tool_seed = (keyed_hash(seed, &["tool_seed"]) >> 32) as u32;
    Recipe::new(
        NonZeroU32::new(tool_seed).unwrap_or(NonZeroU32::MIN),
        palette,
        params,
    )
}

fn keyed_hash(seed: u64, key: &[&str]) -> u64 {
    let mut bytes = seed.to_le_bytes().to_vec();
    for part in key {
        bytes.extend((part.len() as u64).to_le_bytes());
        bytes.extend(part.bytes());
    }
    let fnv = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    let mut z = fnv.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
