use std::borrow::Cow;

use ondas::{HierarchyPath, PathComponent, PathFormatError, PathStyle};

// Commas also need quoting because CLI selection flags accept comma-separated lists.
pub(crate) fn format_path(path: &HierarchyPath) -> String {
    if !path
        .components()
        .any(|component| component.name().contains(','))
    {
        return path.to_string();
    }
    path.components()
        .map(|component| {
            if component.name().contains(',') {
                explicit_logical_component(component)
            } else {
                HierarchyPath::from_components([component]).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorPath {
    base: HierarchyPath,
    range: Option<(u32, u32)>,
    explicit_leaf: bool,
}

impl SelectorPath {
    pub(crate) fn new(base: HierarchyPath, range: Option<(u32, u32)>, explicit_leaf: bool) -> Self {
        Self {
            base,
            range,
            explicit_leaf,
        }
    }

    pub(crate) fn relative_to(&self, scope: &HierarchyPath) -> Option<Self> {
        if scope.is_empty()
            || scope.len() >= self.base.len()
            || !scope
                .components()
                .eq(self.base.components().take(scope.len()))
        {
            return None;
        }
        let base = HierarchyPath::from_components(self.base.components().skip(scope.len()));
        Some(Self::new(base, self.range, self.explicit_leaf))
    }

    pub(crate) fn logical(&self) -> String {
        let base = if self.explicit_leaf {
            let (parent, leaf) = self.split_base();
            join_path(format_path(&parent), &explicit_logical_component(leaf))
        } else {
            format_path(&self.base)
        };
        self.with_range(base)
    }

    pub(crate) fn sv(&self) -> Result<String, PathFormatError> {
        let base = if self.explicit_leaf {
            let (parent, leaf) = self.split_base();
            let name = leaf.name();
            if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_graphic()) {
                return Err(PathFormatError::NotRepresentable {
                    component: name.to_owned(),
                });
            }
            let leaf = match leaf.index() {
                Some(index) => format!("\\{name} [{index}]"),
                None => format!("\\{name} "),
            };
            join_path(parent.format(PathStyle::Sv)?, &leaf)
        } else {
            self.base.format(PathStyle::Sv)?
        };
        Ok(self.with_range(base))
    }

    fn split_base(&self) -> (HierarchyPath, &PathComponent) {
        let leaf = self
            .base
            .components()
            .last()
            .expect("selected declaration has a name");
        let parent =
            HierarchyPath::from_components(self.base.components().take(self.base.len() - 1));
        (parent, leaf)
    }

    fn with_range(&self, mut base: String) -> String {
        if let Some((msb, lsb)) = self.range {
            use std::fmt::Write;
            write!(&mut base, "[{msb}:{lsb}]").expect("writing to String succeeds");
        }
        base
    }
}

fn join_path(mut parent: String, leaf: &str) -> String {
    if !parent.is_empty() {
        parent.push('.');
    }
    parent.push_str(leaf);
    parent
}

fn explicit_logical_component(component: &PathComponent) -> String {
    let name = serde_json::Value::String(component.name().to_owned());
    match component.index() {
        Some(index) => format!("{name}[{index}]"),
        None => name.to_string(),
    }
}
