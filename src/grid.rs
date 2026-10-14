use crate::chassis::round_half_up;

pub(crate) fn size(base: f64, aspect: f64) -> (usize, usize) {
    let width = if aspect > 1.0 {
        round_half_up(base / aspect).max(8.0)
    } else {
        base
    };
    let height = round_half_up(width * aspect).max(8.0);
    (width as usize, height as usize)
}

pub(crate) struct Grid {
    values: Vec<f32>,
    width: usize,
    height: usize,
}

impl Grid {
    pub(crate) fn new(values: Vec<f32>, width: usize, height: usize) -> Self {
        Self {
            values,
            width,
            height,
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn bilinear(&self, gx: f64, gy: f64) -> f64 {
        let (last_x, last_y) = (self.width - 1, self.height - 1);
        let (x0, y0) = ((gx as usize).min(last_x), (gy as usize).min(last_y));
        let (x1, y1) = ((x0 + 1).min(last_x), (y0 + 1).min(last_y));
        let (fx, fy) = (gx - x0 as f64, gy - y0 as f64);
        let (r0, r1) = (y0 * self.width, y1 * self.width);
        let at = |k: usize| f64::from(self.values[k]);
        let a = at(r0 + x0) + (at(r0 + x1) - at(r0 + x0)) * fx;
        let b = at(r1 + x0) + (at(r1 + x1) - at(r1 + x0)) * fx;
        a + (b - a) * fy
    }
}
