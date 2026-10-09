use std::num::NonZeroU32;

use crate::{Params, Recipe, Taste, Tool, sonar};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPin {
    Any,
    Tool(Tool),
    Taste(Taste),
}

pub fn derive(seed: u64, pin: ToolPin) -> Recipe {
    let taste = match pin {
        ToolPin::Any => {
            let slot = keyed_hash(seed, &["tool"]) % Tool::ALL.len() as u64;
            Taste::shipped(Tool::ALL[slot as usize])
        }
        ToolPin::Tool(tool) => Taste::shipped(tool),
        ToolPin::Taste(taste) => taste,
    };
    let tool = taste.tool();
    let draw = |param: &str| keyed_hash(seed, &[tool.slug(), param]);
    let (palette, params) = match tool {
        Tool::Sonar => (sonar::palette(), Params::Sonar(sonar::deal(draw, &taste))),
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

#[cfg(test)]
mod tests {
    use super::{derive, keyed_hash};
    use crate::{Taste, Tool, ToolPin};

    #[test]
    fn each_parameter_draw_hangs_only_on_its_own_key() {
        let taste = Taste::shipped(Tool::Sonar);
        for seed in 0..200 {
            let json: serde_json::Value =
                serde_json::from_str(&derive(seed, ToolPin::Tool(Tool::Sonar)).to_json()).unwrap();
            for param in Tool::Sonar.params() {
                let draw = keyed_hash(seed, &["sonar", param.id]);
                let value = param.deal(taste.bounds(param), draw);
                assert_eq!(
                    json["params"][param.id].as_f64(),
                    Some(value),
                    "seed {seed}: {}",
                    param.id
                );
            }
        }
    }
}
