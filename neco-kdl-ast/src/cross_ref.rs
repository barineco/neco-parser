use crate::NsidPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossRef {
    pub path: NsidPath,
    pub fragment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossRefParseError {
    MultipleHashes,
    EmptyFragment,
    DotInFragment,
    EmptySegment,
}

impl CrossRef {
    pub fn parse(s: &str) -> Result<Self, CrossRefParseError> {
        if s.matches('#').count() > 1 {
            return Err(CrossRefParseError::MultipleHashes);
        }

        match s.split_once('#') {
            None => Ok(Self {
                path: parse_path(s)?,
                fragment: None,
            }),
            Some(("", fragment)) => Ok(Self {
                path: NsidPath::new(),
                fragment: Some(validate_fragment(fragment)?.to_owned()),
            }),
            Some((_, "")) => Err(CrossRefParseError::EmptyFragment),
            Some((path, fragment)) => Ok(Self {
                path: parse_path(path)?,
                fragment: Some(validate_fragment(fragment)?.to_owned()),
            }),
        }
    }

    pub fn from_path(path: NsidPath) -> Self {
        Self {
            path,
            fragment: None,
        }
    }

    pub fn from_path_and_fragment(
        path: NsidPath,
        fragment: String,
    ) -> Result<Self, CrossRefParseError> {
        validate_fragment(&fragment)?;
        Ok(Self {
            path,
            fragment: Some(fragment),
        })
    }

    pub fn nsid(&self) -> &NsidPath {
        &self.path
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    pub fn has_fragment(&self) -> bool {
        self.fragment.is_some()
    }

    pub fn is_local(&self) -> bool {
        self.path.is_empty() && self.fragment.is_some()
    }

    pub fn display(&self) -> String {
        match self.fragment() {
            Some(fragment) if self.path.is_empty() => format!("#{fragment}"),
            Some(fragment) => format!("{}#{fragment}", self.path.display()),
            None => self.path.display(),
        }
    }
}

impl core::fmt::Display for CrossRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.display())
    }
}

fn parse_path(s: &str) -> Result<NsidPath, CrossRefParseError> {
    let path = NsidPath::parse(s);
    path.is_well_formed()
        .then_some(path)
        .ok_or(CrossRefParseError::EmptySegment)
}

fn validate_fragment(fragment: &str) -> Result<&str, CrossRefParseError> {
    if fragment.is_empty() {
        return Err(CrossRefParseError::EmptyFragment);
    }
    if fragment.contains('.') {
        return Err(CrossRefParseError::DotInFragment);
    }
    Ok(fragment)
}

#[cfg(test)]
mod tests {
    use super::{CrossRef, CrossRefParseError};
    use crate::NsidPath;

    #[test]
    fn path_only() {
        let parsed = CrossRef::parse("a.b.c").expect("path ref");
        assert_eq!(parsed.nsid(), &NsidPath::parse("a.b.c"));
        assert_eq!(parsed.fragment(), None);
    }

    #[test]
    fn path_with_fragment() {
        let parsed = CrossRef::parse("a.b.c#frag").expect("fragment ref");
        assert_eq!(parsed.nsid(), &NsidPath::parse("a.b.c"));
        assert_eq!(parsed.fragment(), Some("frag"));
    }

    #[test]
    fn local_only() {
        let parsed = CrossRef::parse("#frag").expect("local ref");
        assert!(parsed.nsid().is_empty());
        assert_eq!(parsed.fragment(), Some("frag"));
    }

    #[test]
    fn multiple_hashes_error() {
        assert_eq!(
            CrossRef::parse("a#b#c"),
            Err(CrossRefParseError::MultipleHashes)
        );
    }

    #[test]
    fn empty_fragment_error() {
        assert_eq!(
            CrossRef::parse("a.b.c#"),
            Err(CrossRefParseError::EmptyFragment)
        );
    }

    #[test]
    fn dot_in_fragment_error() {
        assert_eq!(
            CrossRef::parse("a.b.c#x.y"),
            Err(CrossRefParseError::DotInFragment)
        );
    }

    #[test]
    fn empty_segment_error() {
        assert_eq!(
            CrossRef::parse("a..b"),
            Err(CrossRefParseError::EmptySegment)
        );
    }

    #[test]
    fn round_trip_path_only() {
        let input = "a.b.c";
        assert_eq!(CrossRef::parse(input).expect("parse").display(), input);
    }

    #[test]
    fn round_trip_path_with_fragment() {
        let input = "a.b.c#frag";
        assert_eq!(CrossRef::parse(input).expect("parse").display(), input);
    }

    #[test]
    fn round_trip_local_only() {
        let input = "#frag";
        assert_eq!(CrossRef::parse(input).expect("parse").display(), input);
    }

    #[test]
    fn is_local_for_local_only_ref() {
        assert!(CrossRef::parse("#frag").expect("parse").is_local());
    }

    #[test]
    fn has_fragment_negative_for_path_only() {
        assert!(!CrossRef::parse("a.b.c").expect("parse").has_fragment());
    }
}
