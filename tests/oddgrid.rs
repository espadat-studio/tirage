use tirage::{Frame, Motif, OddgridParams, Params, Recipe, Tool, ToolPin, derive};

const PIN: ToolPin = ToolPin::Tool(Tool::Oddgrid);

fn params(recipe: &Recipe) -> &OddgridParams {
    let Params::Oddgrid(params) = recipe.params() else {
        panic!("not a oddgrid Recipe");
    };
    params
}

fn error<T: std::fmt::Debug>(result: Result<T, tirage::Error>) -> String {
    result.unwrap_err().to_string()
}

#[test]
fn derive_gives_golden_recipes() {
    let golden = [
        (
            1,
            r##"{"tirage":0,"tool":"oddgrid","tool_seed":9611518,"palette":["#ff5e5b","#ffed66","#00cecb","#f2ede4","#9b5de5"],"params":{"cols":69,"scale":13.5,"density":0.92,"grain":1.21,"variety":0.6,"block":0.48,"bsize":6,"speck":0.14,"motifs":"Square","motifAmt":0.42,"markSize":0.99,"balance":-0.95,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"oddgrid","tool_seed":3724278653,"palette":["#ff5e5b","#ffed66","#00cecb","#f2ede4","#9b5de5"],"params":{"cols":71,"scale":14.0,"density":0.74,"grain":1.09,"variety":1.37,"block":0.57,"bsize":12,"speck":0.39,"motifs":"Dot","motifAmt":0.96,"markSize":0.61,"balance":0.35,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"oddgrid","tool_seed":3565986084,"palette":["#ff5e5b","#ffed66","#00cecb","#f2ede4","#9b5de5"],"params":{"cols":108,"scale":45.5,"density":0.58,"grain":0.45,"variety":0.1,"block":0.68,"bsize":8,"speck":0.45,"motifs":"Ring","motifAmt":0.41,"markSize":0.39,"balance":-1.05,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PIN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut seen = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, PIN);
        let p = params(&recipe);
        for (value, min, max, unit) in [
            (f64::from(p.cols()), 12.0, 110.0, 1.0),
            (p.scale(), 2.0, 60.0, 10.0),
            (p.density(), 0.4, 1.0, 100.0),
            (p.grain(), 0.0, 1.4, 100.0),
            (p.variety(), 0.0, 1.6, 100.0),
            (p.block(), 0.0, 1.0, 100.0),
            (f64::from(p.bsize()), 2.0, 16.0, 1.0),
            (p.speck(), 0.0, 0.45, 100.0),
            (p.motif_amt(), 0.0, 1.0, 100.0),
            (p.mark_size(), 0.15, 1.0, 100.0),
            (p.balance(), -1.2, 1.2, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !seen.contains(&p.motifs()) {
            seen.push(p.motifs());
        }
    }
    assert_eq!(seen.len(), Motif::ALL.len());
}

#[test]
fn oddgrid_is_a_still() {
    assert_eq!(Tool::Oddgrid.frames(), 1);
    let recipe = derive(7, PIN);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, PIN);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = OddgridParams::default();
    assert_eq!(
        error(p.set_cols(111)),
        "oddgrid: cols 111 is outside 12..=110"
    );
    assert_eq!(p, OddgridParams::default());
}
