use crate::{Error, Palette, Tool};

macro_rules! tools {
    ($($variant:ident: $module:ident::{$params:ident $(, $extra:ident)* $(,)?}),* $(,)?) => {
        $(mod $module;)*
        $(pub use $module::{$params $(, $extra)*};)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Tool {
            $($variant,)*
        }

        impl Tool {
            pub const ALL: &[Tool] = &[$(Tool::$variant,)*];

            pub fn slug(self) -> &'static str {
                match self {
                    $(Self::$variant => $module::SLUG,)*
                }
            }

            pub fn frames(self) -> u32 {
                match self {
                    $(Self::$variant => $module::FRAMES,)*
                }
            }

            pub fn fps(self) -> u32 {
                match self {
                    $(Self::$variant => $module::FPS,)*
                }
            }

            pub fn parameters(self) -> Vec<$crate::Parameter> {
                match self {
                    $(Self::$variant => $module::parameters(),)*
                }
            }

            pub(crate) fn params(self) -> &'static [$crate::param::Param] {
                match self {
                    $(Self::$variant => $module::PARAMS,)*
                }
            }

            pub(crate) fn max_inks(self) -> Option<usize> {
                match self {
                    $(Self::$variant => $module::MAX_INKS,)*
                }
            }

            pub(crate) fn palette(self) -> Option<$crate::Palette> {
                match self {
                    $(Self::$variant => $module::palette().into(),)*
                }
            }

            pub(crate) fn deal(self, draw: impl Fn(&str) -> u64, taste: &$crate::Taste) -> Params {
                match self {
                    $(Self::$variant => Params::$variant($module::deal(draw, taste)),)*
                }
            }

            pub(crate) fn params_from_json(
                self,
                json: serde_json::Value,
            ) -> Result<Params, $crate::Error> {
                match self {
                    $(Self::$variant => Ok(Params::$variant($module::from_json(json)?)),)*
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        #[serde(untagged)]
        pub enum Params {
            $($variant($params),)*
        }

        impl Params {
            pub fn tool(&self) -> Tool {
                match self {
                    $(Self::$variant(_) => Tool::$variant,)*
                }
            }

            pub(crate) fn render(
                &self,
                surface: &mut $crate::surface::Surface,
                palette: Option<&$crate::Palette>,
                tool_seed: u32,
                t: u32,
            ) {
                use $crate::registry::Inks;
                match self {
                    $(Self::$variant(params) => $module::render(surface, params, Inks::inks(palette), tool_seed, t),)*
                }
            }
        }
    };
}

pub(crate) use tools;

impl Tool {
    pub fn from_slug(slug: &str) -> Result<Self, Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|tool| tool.slug() == slug)
            .ok_or_else(|| Error::UnknownTool(slug.to_owned()))
    }
}

pub(crate) trait Inks<'a> {
    fn inks(palette: Option<&'a Palette>) -> Self;
}

impl<'a> Inks<'a> for &'a Palette {
    fn inks(palette: Option<&'a Palette>) -> Self {
        palette.expect("a Recipe holds a Palette for a Tool that takes one")
    }
}

impl<'a> Inks<'a> for Option<&'a Palette> {
    fn inks(palette: Option<&'a Palette>) -> Self {
        palette
    }
}
