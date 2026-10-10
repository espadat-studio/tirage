use tirage::{Error, MAX_EDGE, Tool};

pub fn for_error(error: &Error) -> Option<String> {
    let hint = match error {
        Error::Major(_) => {
            "derive it again from its Seed with this build: tirage derive --seed <N>".to_owned()
        }
        Error::UnknownTool(slug) => match closest(slug, Tool::ALL.iter().map(|tool| tool.slug())) {
            Some(slug) => format!("did you mean '{slug}'?"),
            None => "run 'tirage tools' to list Tools".to_owned(),
        },
        Error::UnknownParameter { tool, param } => {
            match closest(param, tool.parameters().iter().map(|p| p.id)) {
                Some(id) => format!("did you mean '{id}'?"),
                None => format!("run 'tirage tools' to list {}'s Parameters", tool.slug()),
            }
        }
        Error::OutOfRange { .. } | Error::OffStep { .. } | Error::Reversed { .. } => {
            "run 'tirage tools' to see each Parameter's range and step".to_owned()
        }
        Error::Ink(_) | Error::FewInks(_) => {
            "pass at least 2 comma-separated #rrggbb inks, like '#000000,#ffffff'".to_owned()
        }
        Error::NoPalette(tool) => format!(
            "{} draws its own colours; drop --palette, or pin a Tool that takes a Palette with --tool <slug>",
            tool.slug()
        ),
        Error::TooManyInks { tool, max, .. } => {
            format!("pass at most {max} inks for {}", tool.slug())
        }
        Error::FrameSize { .. } => {
            format!("pass --size <W>x<H> with each edge in 1..={MAX_EDGE}, like 1080x1920")
        }
        Error::FrameTime { frames, .. } => format!(
            "pass --frame from 0 to {}, or leave it out for frame 0",
            frames - 1
        ),
        Error::Json(_) => "pass a Recipe printed by 'tirage derive'".to_owned(),
        Error::TasteJson(_) => {
            "Taste bounds are a JSON object of a tool and [min, max] per Parameter id".to_owned()
        }
        _ => return None,
    };
    Some(hint)
}

#[cfg(feature = "encode")]
pub fn for_encode_error(error: &tirage_encode::Error) -> Option<String> {
    match error {
        tirage_encode::Error::OverCap { .. } => {
            Some("try another Seed, or a simpler Recipe".to_owned())
        }
        _ => None,
    }
}

fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .map(|candidate| (distance(word, candidate), candidate))
        .filter(|&(d, candidate)| d <= 2 && d < candidate.len())
        .min_by_key(|&(d, _)| d)
        .map(|(_, candidate)| candidate)
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &y) in b.iter().enumerate() {
            let substitute = diagonal + usize::from(x != y);
            diagonal = row[j + 1];
            row[j + 1] = substitute.min(row[j] + 1).min(diagonal + 1);
        }
    }
    row[b.len()]
}
