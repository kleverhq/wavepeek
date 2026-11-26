use std::borrow::Cow;
use std::fmt::Write;

use ondas::PathComponent;

pub(super) fn component_order(component: &PathComponent) -> (Cow<'_, str>, &PathComponent) {
    (component_spelling(component), component)
}

pub(super) fn component_spelling(component: &PathComponent) -> Cow<'_, str> {
    let name = component.name();
    let indices = component.indices();
    if indices.is_empty() {
        return Cow::Borrowed(name);
    }

    let mut spelling = name.to_owned();
    for index in indices {
        write!(spelling, "[{index}]").expect("writing to a string should succeed");
    }
    Cow::Owned(spelling)
}
