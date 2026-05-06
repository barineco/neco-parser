use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NsidPath {
    segments: Vec<String>,
}

impl NsidPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(s: &str) -> Self {
        if s.is_empty() {
            return Self::new();
        }
        Self::from_segments(s.split('.'))
    }

    pub fn from_segments<I, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            segments: iter.into_iter().map(Into::into).collect(),
        }
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn first(&self) -> Option<&str> {
        self.segments.first().map(String::as_str)
    }

    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    pub fn parent(&self) -> Option<Self> {
        let parent_len = self.segments.len().checked_sub(1)?;
        Some(Self {
            segments: self.segments[..parent_len].to_vec(),
        })
    }

    pub fn child(&self, segment: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.into());
        Self { segments }
    }

    pub fn starts_with(&self, other: &NsidPath) -> bool {
        self.segments.starts_with(other.segments())
    }

    pub fn display(&self) -> String {
        self.segments.join(".")
    }

    pub fn is_well_formed(&self) -> bool {
        self.segments
            .iter()
            .all(|s| !s.is_empty() && !s.contains('.') && !s.contains('#'))
    }

    pub fn to_fs_path(&self, ext: &str) -> PathBuf {
        let Some((leaf, parents)) = self.segments.split_last() else {
            return PathBuf::new();
        };
        let mut path = PathBuf::new();
        for segment in parents {
            path.push(segment);
        }
        let ext = ext.strip_prefix('.').unwrap_or(ext);
        if ext.is_empty() {
            path.push(leaf);
        } else {
            path.push(format!("{leaf}.{ext}"));
        }
        path
    }

    pub fn from_fs_path(path: &Path, base: &Path) -> Option<Self> {
        let relative = path.strip_prefix(base).ok()?;
        let mut segments = Vec::new();
        for component in relative.components() {
            match component {
                Component::Normal(part) => segments.push(part.to_str()?.to_owned()),
                _ => return None,
            }
        }
        let leaf = segments.last_mut()?;
        let stem = Path::new(leaf).file_stem()?.to_str()?.to_owned();
        *leaf = stem;

        let path = Self { segments };
        path.is_well_formed().then_some(path)
    }
}

impl core::fmt::Display for NsidPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::NsidPath;
    use std::path::Path;

    #[test]
    fn segment_with_dot_not_well_formed() {
        let path = NsidPath::from_segments(["a.b"]);
        assert!(!path.is_well_formed());
    }

    #[test]
    fn segment_with_hash_not_well_formed() {
        let path = NsidPath::from_segments(["a#b"]);
        assert!(!path.is_well_formed());
    }

    #[test]
    fn empty_segment_not_well_formed() {
        let path = NsidPath::from_segments(["a", "", "b"]);
        assert!(!path.is_well_formed());
    }

    #[test]
    fn empty_string_yields_empty_path() {
        assert!(NsidPath::parse("").is_empty());
    }

    #[test]
    fn dot_separator_only() {
        assert_eq!(
            NsidPath::parse("a.b.c").segments(),
            &["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn round_trip() {
        let input = "a.b.c";
        assert_eq!(NsidPath::parse(input).display(), input);
    }

    #[test]
    fn parent_of_three_segment() {
        assert_eq!(
            NsidPath::parse("a.b.c").parent(),
            Some(NsidPath::parse("a.b"))
        );
    }

    #[test]
    fn parent_of_empty() {
        assert_eq!(NsidPath::new().parent(), None);
    }

    #[test]
    fn child_appends_segment() {
        assert_eq!(NsidPath::parse("a.b").child("c"), NsidPath::parse("a.b.c"));
    }

    #[test]
    fn starts_with_prefix() {
        assert!(NsidPath::parse("a.b.c").starts_with(&NsidPath::parse("a.b")));
    }

    #[test]
    fn starts_with_not_prefix() {
        assert!(!NsidPath::parse("a.b.c").starts_with(&NsidPath::parse("a.c")));
    }

    #[test]
    fn to_fs_path_with_ext() {
        assert_eq!(
            NsidPath::parse("a.b.c").to_fs_path("kdl"),
            Path::new("a").join("b").join("c.kdl")
        );
    }

    #[test]
    fn to_fs_path_empty_returns_just_ext_or_empty() {
        assert_eq!(NsidPath::new().to_fs_path("kdl"), std::path::PathBuf::new());
        assert_eq!(NsidPath::new().to_fs_path(""), std::path::PathBuf::new());
    }

    #[test]
    fn from_fs_path_relative() {
        assert_eq!(
            NsidPath::from_fs_path(Path::new("base/a/b/c.kdl"), Path::new("base")),
            Some(NsidPath::parse("a.b.c"))
        );
    }

    #[test]
    fn from_fs_path_outside_base() {
        assert_eq!(
            NsidPath::from_fs_path(Path::new("other/a.kdl"), Path::new("base")),
            None
        );
    }

    #[test]
    fn from_fs_path_with_dot_segment_returns_none() {
        assert_eq!(
            NsidPath::from_fs_path(Path::new("base/.config/file.kdl"), Path::new("base")),
            None
        );
    }
}
