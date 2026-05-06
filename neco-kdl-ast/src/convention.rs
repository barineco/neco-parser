#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Convention {
    pub markers: Vec<Marker>,
}

impl Convention {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_marker(mut self, m: Marker) -> Self {
        self.markers.push(m);
        self
    }

    pub fn is_marker_kind(&self, node_name: &str) -> bool {
        self.markers.iter().any(|marker| match marker {
            Marker::Kind(kind) => node_name == kind,
            Marker::Prefix(prefix) => node_name.starts_with(*prefix),
        })
    }

    pub fn strip_marker_prefix<'a>(&self, node_name: &'a str) -> &'a str {
        for marker in &self.markers {
            if let Marker::Prefix(prefix) = marker {
                if let Some(stripped) = node_name.strip_prefix(*prefix) {
                    return stripped;
                }
            }
        }
        node_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Marker {
    Kind(String),
    Prefix(char),
}

#[cfg(test)]
mod tests {
    use super::{Convention, Marker};

    #[test]
    fn convention_is_marker_kind_kind() {
        let conv = Convention::new().with_marker(Marker::Kind("_meta".to_owned()));
        assert!(conv.is_marker_kind("_meta"));
    }

    #[test]
    fn convention_is_marker_kind_prefix() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        assert!(conv.is_marker_kind(":meta"));
    }

    #[test]
    fn convention_strip_marker_prefix_kind() {
        let conv = Convention::new().with_marker(Marker::Kind("_meta".to_owned()));
        assert_eq!(conv.strip_marker_prefix("_meta"), "_meta");
    }

    #[test]
    fn convention_strip_marker_prefix_prefix() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        assert_eq!(conv.strip_marker_prefix(":meta"), "meta");
    }
}
