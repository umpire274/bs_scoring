pub(crate) fn parse_outcome_json(raw: Option<&str>) -> Option<serde_json::Value> {
    raw.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
}

pub(crate) fn get_sequence(v: &serde_json::Value) -> Option<&str> {
    v.get("sequence").and_then(|x| x.as_str())
}

pub(crate) fn get_fielder(v: &serde_json::Value) -> Option<u64> {
    v.get("fielder").and_then(|x| x.as_u64())
}

pub(crate) fn get_foul_flag(v: &serde_json::Value) -> bool {
    v.get("in_foul_territory")
        .and_then(|x| x.as_bool())
        .unwrap_or(false)
}