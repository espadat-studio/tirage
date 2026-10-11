use tirage::{Error, Tool, ToolPin, derive, render_svg};

#[test]
fn every_slug_round_trips_through_from_slug() {
    for &tool in Tool::ALL {
        assert_eq!(Tool::from_slug(tool.slug()).unwrap(), tool, "{tool:?}");
    }
}

#[test]
fn all_lists_each_tool_exactly_once() {
    for &tool in Tool::ALL {
        let listed = Tool::ALL.iter().filter(|&&t| t == tool).count();
        assert_eq!(listed, 1, "{tool:?}");
    }
}

#[test]
fn render_svg_refuses_a_tool_without_svg() {
    let recipe = derive(1, ToolPin::Tool(Tool::Husk));
    assert_eq!(
        render_svg(&recipe, 540, 960),
        Err(Error::SvgUnsupported { tool: Tool::Husk })
    );
    assert_eq!(
        Error::SvgUnsupported { tool: Tool::Husk }.to_string(),
        "husk: renders PNG only, not SVG"
    );
}
