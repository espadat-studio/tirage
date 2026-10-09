use crate::{Error, Parameter};

pub(crate) struct Param {
    pub(crate) scope: &'static str,
    pub(crate) id: &'static str,
    pub(crate) min: u32,
    pub(crate) max: u32,
    pub(crate) step: u32,
    pub(crate) unit: u32,
    pub(crate) taste: (u32, u32),
}

impl Param {
    pub(crate) const fn new(
        scope: &'static str,
        id: &'static str,
        min: u32,
        max: u32,
        unit: u32,
    ) -> Self {
        Self {
            scope,
            id,
            min,
            max,
            step: 1,
            unit,
            taste: (min, max),
        }
    }

    pub(crate) fn deal(&self, draw: u64) -> f64 {
        let (low, high) = self.taste;
        let steps = u64::from((high - low) / self.step + 1);
        let ticks = low + (draw % steps) as u32 * self.step;
        f64::from(ticks) / f64::from(self.unit)
    }

    pub(crate) fn parameter(&self) -> Parameter {
        let unit = f64::from(self.unit);
        Parameter {
            id: self.id,
            min: f64::from(self.min) / unit,
            max: f64::from(self.max) / unit,
            step: f64::from(self.step) / unit,
        }
    }

    pub(crate) fn check(&self, value: f64) -> Result<f64, Error> {
        let Parameter { min, max, .. } = self.parameter();
        if (min..=max).contains(&value) {
            return Ok(value);
        }
        Err(Error::OutOfRange {
            scope: self.scope,
            param: self.id,
            value,
            min,
            max,
        })
    }
}
