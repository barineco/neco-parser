use crate::{AxisForm, Convention, CrossRef, Marker, NsidPath, PropertyChildForm, StructuredNode};
use neco_ast::StructuredDocument;
use neco_kdl::{parse as kdl_parse, KdlDocument, KdlError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) document: KdlDocument,
}

impl<'a> StructuredDocument<'a> for Document {
    type Node = StructuredNode<'a>;

    fn nodes(&'a self) -> Vec<Self::Node> {
        self.structured_nodes().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Strict1To1,
    Bundle,
    CratisDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutViolation {
    pub kind: LayoutViolationKind,
    pub file_path: PathBuf,
    pub entity_nsid: Option<NsidPath>,
    pub expected_prefix: Option<NsidPath>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutViolationKind {
    StrictMismatch,
    BundlePrefixMismatch,
    IllegalSegment,
    PathOutsideBase,
    StrictMultipleEntities,
}

impl Document {
    pub fn from_kdl(document: KdlDocument) -> Self {
        Self { document }
    }

    pub fn parse(input: &str) -> Result<Self, KdlError> {
        kdl_parse(input).map(Self::from_kdl)
    }

    pub fn structured_nodes(&self) -> impl Iterator<Item = StructuredNode<'_>> {
        self.document.nodes().iter().map(StructuredNode::from_kdl)
    }

    pub fn find_by_kind(&self, kind: &NsidPath) -> impl Iterator<Item = StructuredNode<'_>> + '_ {
        let kind = kind.clone();
        self.structured_nodes()
            .filter(move |node| node.node_name_as_nsid() == kind)
    }

    pub fn find_by_kind_and_identifier(
        &self,
        kind: &NsidPath,
        identifier: &NsidPath,
    ) -> Option<StructuredNode<'_>> {
        self.find_by_kind(kind)
            .find(|node| node.structured_name().identifier == *identifier)
    }

    pub fn find_by_identifier(&self, identifier: &NsidPath) -> Option<StructuredNode<'_>> {
        self.structured_nodes()
            .find(|node| node.structured_name().identifier == *identifier)
    }

    /// Convention の per-axis 正規形宣言に従って軸 1-5 の expand / collapse を crate 規定順序で適用。
    ///
    /// 軸間順序は `軸 5 ( kind keyword ) → 軸 4 ( type annotation ) → 軸 1 ( namespace ) →
    /// 軸 2 ( procedure ) → 軸 3 ( property-child )` で固定 ( marker 境界保護順 )。
    /// 軸 ごとの `AxisForm::Off` / `PropertyChildForm::Off` は no-op。
    /// 軸 4 / 5 で plain `Expand` / `Collapse` ( marker なし ) は no-op、
    /// 軸 1 / 2 で `*WithMarker` variant も no-op。
    /// 軸 2 ( procedure ) は Convention の `Marker::Kind` を per-kind dot-chain target として再利用する。
    pub fn render_as(&self, conv: &Convention) -> Document {
        let mut doc = self.clone();
        doc = apply_axis_5(doc, &conv.kind_keyword_form, conv);
        doc = apply_axis_4(doc, &conv.type_annotation_form, conv);
        doc = apply_axis_1(doc, &conv.namespace_form, conv);
        doc = apply_axis_2(doc, &conv.procedure_form, conv);
        doc = apply_axis_3(doc, &conv.property_child_form, conv);
        doc
    }

    pub fn resolve(&self, cross_ref: &CrossRef) -> Option<StructuredNode<'_>> {
        match (cross_ref.is_local(), cross_ref.fragment()) {
            (true, Some(fragment)) => self.find_by_identifier(&NsidPath::parse(fragment)),
            (false, None) => self.find_by_identifier(cross_ref.nsid()),
            (false, Some(fragment)) => {
                let entity = self.find_by_identifier(cross_ref.nsid())?;
                find_fragment(entity, fragment)
            }
            _ => None,
        }
    }

    pub fn verify_layout(
        &self,
        mode: LayoutMode,
        file_path: &Path,
        base: &Path,
    ) -> Vec<LayoutViolation> {
        if matches!(mode, LayoutMode::CratisDir) {
            return Vec::new();
        }

        let Some(expected) = fs_nsid_or_violation(file_path, base) else {
            return vec![LayoutViolation {
                kind: LayoutViolationKind::PathOutsideBase,
                file_path: file_path.to_owned(),
                entity_nsid: None,
                expected_prefix: None,
                message: "file path is outside base".to_owned(),
            }];
        };

        let Some(expected) = expected else {
            return vec![LayoutViolation {
                kind: LayoutViolationKind::IllegalSegment,
                file_path: file_path.to_owned(),
                entity_nsid: None,
                expected_prefix: None,
                message: "file path contains an illegal namespace segment".to_owned(),
            }];
        };

        match mode {
            LayoutMode::Strict1To1 => self.verify_strict_layout(file_path, expected),
            LayoutMode::Bundle => self.verify_bundle_layout(file_path, expected),
            LayoutMode::CratisDir => Vec::new(),
        }
    }

    pub fn as_kdl(&self) -> &KdlDocument {
        &self.document
    }

    fn verify_strict_layout(&self, file_path: &Path, expected: NsidPath) -> Vec<LayoutViolation> {
        let nodes: Vec<_> = self.structured_nodes().collect();
        let mut violations = Vec::new();
        if nodes.len() > 1 {
            violations.push(LayoutViolation {
                kind: LayoutViolationKind::StrictMultipleEntities,
                file_path: file_path.to_owned(),
                entity_nsid: None,
                expected_prefix: Some(expected.clone()),
                message: "strict layout accepts exactly one top-level entity".to_owned(),
            });
        }
        if let Some(node) = nodes.first() {
            let actual = node.structured_name().identifier;
            if actual != expected {
                violations.push(LayoutViolation {
                    kind: LayoutViolationKind::StrictMismatch,
                    file_path: file_path.to_owned(),
                    entity_nsid: Some(actual),
                    expected_prefix: Some(expected),
                    message: "top-level entity does not match file path".to_owned(),
                });
            }
        }
        violations
    }

    fn verify_bundle_layout(&self, file_path: &Path, expected: NsidPath) -> Vec<LayoutViolation> {
        self.structured_nodes()
            .filter_map(|node| {
                let actual = node.structured_name().identifier;
                let strict_prefix = actual.starts_with(&expected) && actual.len() > expected.len();
                (!strict_prefix).then(|| LayoutViolation {
                    kind: LayoutViolationKind::BundlePrefixMismatch,
                    file_path: file_path.to_owned(),
                    entity_nsid: Some(actual),
                    expected_prefix: Some(expected.clone()),
                    message: "top-level entity does not include file path as strict prefix"
                        .to_owned(),
                })
            })
            .collect()
    }
}

fn fs_nsid_or_violation(file_path: &Path, base: &Path) -> Option<Option<NsidPath>> {
    file_path.strip_prefix(base).ok()?;
    Some(NsidPath::from_fs_path(file_path, base))
}

fn find_fragment<'a>(entity: StructuredNode<'a>, fragment: &str) -> Option<StructuredNode<'a>> {
    let fragment_nsid = NsidPath::parse(fragment);
    for child in entity.children() {
        if child.node_name_as_nsid() == fragment_nsid {
            return Some(child);
        }
    }
    find_child_identifier(entity, &fragment_nsid)
}

fn find_child_identifier<'a>(
    node: StructuredNode<'a>,
    identifier: &NsidPath,
) -> Option<StructuredNode<'a>> {
    for child in node.children() {
        if child.node_name_as_nsid() == *identifier
            || child.structured_name().identifier == *identifier
        {
            return Some(child);
        }
        if let Some(found) = find_child_identifier(child, identifier) {
            return Some(found);
        }
    }
    None
}

fn apply_axis_1(doc: Document, form: &AxisForm, conv: &Convention) -> Document {
    match form {
        AxisForm::Off | AxisForm::ExpandWithMarker(_) | AxisForm::CollapseWithMarker(_) => doc,
        AxisForm::Expand => doc.nest(conv).0,
        AxisForm::Collapse => doc.flatten(conv).0,
    }
}

fn apply_axis_2(doc: Document, form: &AxisForm, conv: &Convention) -> Document {
    match form {
        AxisForm::Off | AxisForm::ExpandWithMarker(_) | AxisForm::CollapseWithMarker(_) => doc,
        AxisForm::Expand => kind_markers(conv).into_iter().fold(doc, |d, k| {
            d.expand_dot_chain(&k, conv).0
        }),
        AxisForm::Collapse => kind_markers(conv).into_iter().fold(doc, |d, k| {
            d.collapse_dot_chain(&k, conv).0
        }),
    }
}

fn apply_axis_3(doc: Document, form: &PropertyChildForm, conv: &Convention) -> Document {
    match form {
        PropertyChildForm::Off => doc,
        PropertyChildForm::Expand => doc.expand_properties(conv).0,
        PropertyChildForm::Collapse => doc.collapse_properties(conv).0,
    }
}

fn apply_axis_4(doc: Document, form: &AxisForm, conv: &Convention) -> Document {
    match form {
        AxisForm::Off | AxisForm::Expand | AxisForm::Collapse => doc,
        AxisForm::ExpandWithMarker(m) => doc.expand_type_annotations(m, conv).0,
        AxisForm::CollapseWithMarker(m) => doc.collapse_type_annotations(m, conv).0,
    }
}

fn apply_axis_5(doc: Document, form: &AxisForm, conv: &Convention) -> Document {
    match form {
        AxisForm::Off | AxisForm::Expand | AxisForm::Collapse => doc,
        AxisForm::ExpandWithMarker(m) => doc.expand_kind_keyword(m, conv).0,
        AxisForm::CollapseWithMarker(m) => doc.collapse_kind_keyword(m, conv).0,
    }
}

fn kind_markers(conv: &Convention) -> Vec<String> {
    conv.markers
        .iter()
        .filter_map(|marker| match marker {
            Marker::Kind(k) => Some(k.clone()),
            Marker::Prefix(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Document, LayoutMode, LayoutViolationKind};
    use crate::{CrossRef, NsidPath};
    use std::path::Path;

    #[test]
    fn parse_basic() {
        let doc = Document::parse("entity \"alpha\"\n").expect("parse");
        assert_eq!(doc.as_kdl().nodes().len(), 1);
    }

    #[test]
    fn find_by_kind_returns_matching() {
        let doc = Document::parse("entity \"alpha\"\nother \"beta\"\n").expect("parse");
        let found: Vec<_> = doc.find_by_kind(&NsidPath::parse("entity")).collect();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn find_by_kind_and_identifier_match() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        assert!(doc
            .find_by_kind_and_identifier(&NsidPath::parse("entity"), &NsidPath::parse("alpha.beta"))
            .is_some());
    }

    #[test]
    fn find_by_identifier_match() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        assert!(doc
            .find_by_identifier(&NsidPath::parse("alpha.beta"))
            .is_some());
    }

    #[test]
    fn resolve_path_only() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        let resolved = doc.resolve(&CrossRef::parse("alpha.beta").expect("ref"));
        assert!(resolved.is_some());
    }

    #[test]
    fn resolve_local_only() {
        let doc = Document::parse("item \"frag\"\n").expect("parse");
        let resolved = doc.resolve(&CrossRef::parse("#frag").expect("ref"));
        assert!(resolved.is_some());
    }

    #[test]
    fn resolve_path_with_fragment() {
        let doc =
            Document::parse("entity \"alpha.beta\" { defs { frag \"value\" } }\n").expect("parse");
        let resolved = doc.resolve(&CrossRef::parse("alpha.beta#frag").expect("ref"));
        assert!(resolved.is_some());
    }

    #[test]
    fn resolve_unresolvable() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        let resolved = doc.resolve(&CrossRef::parse("missing").expect("ref"));
        assert!(resolved.is_none());
    }

    #[test]
    fn axiom_a5b_strict_match() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Strict1To1,
            Path::new("base/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn axiom_a5b_strict_mismatch() {
        let doc = Document::parse("entity \"other.path\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Strict1To1,
            Path::new("base/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert_eq!(violations[0].kind, LayoutViolationKind::StrictMismatch);
    }

    #[test]
    fn axiom_a5b_strict_multiple_entities() {
        let doc =
            Document::parse("entity \"alpha.beta\"\nentity \"alpha.gamma\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Strict1To1,
            Path::new("base/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert!(violations
            .iter()
            .any(|v| v.kind == LayoutViolationKind::StrictMultipleEntities));
    }

    #[test]
    fn axiom_a5c_bundle_match() {
        let doc = Document::parse("entity \"alpha.beta.one\"\nentity \"alpha.beta.two\"\n")
            .expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Bundle,
            Path::new("base/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn axiom_a5c_bundle_prefix_mismatch() {
        let doc = Document::parse("entity \"other.path\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Bundle,
            Path::new("base/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert_eq!(
            violations[0].kind,
            LayoutViolationKind::BundlePrefixMismatch
        );
    }

    #[test]
    fn axiom_a5d_cratis_skip() {
        let doc = Document::parse("entity \"other.path\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::CratisDir,
            Path::new("outside/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn verify_layout_path_outside_base() {
        let doc = Document::parse("entity \"alpha.beta\"\n").expect("parse");
        let violations = doc.verify_layout(
            LayoutMode::Strict1To1,
            Path::new("outside/alpha/beta.kdl"),
            Path::new("base"),
        );
        assert_eq!(violations[0].kind, LayoutViolationKind::PathOutsideBase);
    }
}
