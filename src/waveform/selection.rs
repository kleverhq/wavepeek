use std::borrow::Cow;

use ondas::PathComponent;

pub(super) fn component_order(component: &PathComponent) -> (Cow<'_, str>, &PathComponent) {
    (component_spelling(component), component)
}

pub(super) fn component_spelling(component: &PathComponent) -> Cow<'_, str> {
    let name = component.name();
    match component.index() {
        Some(index) => Cow::Owned(format!("{name}[{index}]")),
        None => Cow::Borrowed(name),
    }
}
