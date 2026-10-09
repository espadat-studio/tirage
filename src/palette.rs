use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette(Vec<[u8; 3]>);

impl Palette {
    pub fn from_hex<S: AsRef<str>>(inks: &[S]) -> Result<Self, Error> {
        if inks.len() < 2 {
            return Err(Error::FewInks(inks.len()));
        }
        inks.iter()
            .map(|ink| parse_ink(ink.as_ref()))
            .collect::<Result<_, _>>()
            .map(Self)
    }

    pub fn to_hex(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|[r, g, b]| format!("#{r:02x}{g:02x}{b:02x}"))
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn ink(&self, slot: usize) -> [u8; 3] {
        self.0[slot % self.0.len()]
    }
}

fn parse_ink(ink: &str) -> Result<[u8; 3], Error> {
    let digits = ink
        .strip_prefix('#')
        .filter(|digits| digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| Error::Ink(ink.to_owned()))?;
    let [_, r, g, b] = u32::from_str_radix(digits, 16).unwrap().to_be_bytes();
    Ok([r, g, b])
}
