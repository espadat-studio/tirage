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

    pub(crate) fn deal(&self, (low, high): (u32, u32), draw: u64) -> f64 {
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

    pub(crate) fn ticks(&self, value: f64) -> Result<u32, Error> {
        self.check(value)?;
        let exact = value * f64::from(self.unit);
        let ticks = exact.round();
        if (exact - ticks).abs() > 1e-9 || !(ticks as u32 - self.min).is_multiple_of(self.step) {
            let Parameter { min, max, step, .. } = self.parameter();
            return Err(Error::OffStep {
                scope: self.scope,
                param: self.id,
                value,
                min,
                max,
                step,
            });
        }
        Ok(ticks as u32)
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
