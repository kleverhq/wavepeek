// Commas also need quoting because CLI selection flags accept comma-separated lists.
pub(crate) fn format_path(path: &ondas::HierarchyPath) -> String {
    if !path.components().any(|component| component.contains(',')) {
        return path.to_string();
    }
    path.components()
        .map(|component| {
            if component.contains(',') {
                serde_json::Value::String(component.to_owned()).to_string()
            } else {
                ondas::HierarchyPath::from_components([component]).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}
