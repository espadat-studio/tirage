use tirage::Tool;

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
