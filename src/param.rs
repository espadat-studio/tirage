use crate::{Error, Parameter, ParameterKind};

pub(crate) struct Param {
    pub(crate) scope: &'static str,
    pub(crate) id: &'static str,
    pub(crate) min: i32,
    pub(crate) max: i32,
    pub(crate) step: u32,
    pub(crate) unit: u32,
    pub(crate) taste: (i32, i32),
}

impl Param {
    pub(crate) const fn new(
        scope: &'static str,
        id: &'static str,
        min: i32,
        max: i32,
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

    pub(crate) fn deal(&self, (low, high): (i32, i32), draw: u64) -> f64 {
        let steps = u64::from(high.abs_diff(low) / self.step + 1);
        let ticks = low + ((draw % steps) as u32 * self.step) as i32;
        f64::from(ticks) / f64::from(self.unit)
    }

    pub(crate) fn parameter(&self) -> Parameter {
        let (min, max, step) = self.range();
        Parameter {
            id: self.id,
            kind: ParameterKind::Range { min, max, step },
        }
    }

    fn range(&self) -> (f64, f64, f64) {
        let unit = f64::from(self.unit);
        (
            f64::from(self.min) / unit,
            f64::from(self.max) / unit,
            f64::from(self.step) / unit,
        )
    }

    pub(crate) fn ticks(&self, value: f64) -> Result<i32, Error> {
        self.check(value)?;
        let exact = value * f64::from(self.unit);
        let ticks = exact.round();
        if (exact - ticks).abs() > 1e-9
            || !(ticks as i32).abs_diff(self.min).is_multiple_of(self.step)
        {
            let (min, max, step) = self.range();
            return Err(Error::OffStep {
                scope: self.scope,
                param: self.id,
                value,
                min,
                max,
                step,
            });
        }
        Ok(ticks as i32)
    }

    pub(crate) fn check(&self, value: f64) -> Result<f64, Error> {
        let (min, max, _) = self.range();
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

#[cfg(test)]
mod tests {
    use super::Param;

    const DIR: Param = Param::new("test", "dir", -100, 100, 100);

    #[test]
    fn a_range_below_zero_deals_and_parses_on_its_grid() {
        assert_eq!(DIR.deal((-100, 100), 0), -1.0);
        assert_eq!(DIR.deal((-100, 100), 200), 1.0);
        assert_eq!(DIR.deal((-100, 100), 201), -1.0);
        assert_eq!(DIR.ticks(-0.35).unwrap(), -35);
        assert!(DIR.ticks(-1.01).is_err());
    }
}
