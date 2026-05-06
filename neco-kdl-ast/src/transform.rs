use crate::{Convention, Document, Marker};
use neco_kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformOutcome {
    Applied,
    NoOp { reason: String },
}

impl Document {
    pub fn nest(self, conv: &Convention) -> (Self, TransformOutcome) {
        let (nodes, changed) = transform_nodes(self.document.nodes, conv, false, |node, skip| {
            let skip = skip || conv.is_marker_kind(&node.name);
            nest_node(node, skip)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no dotted namespace node"),
        )
    }

    pub fn flatten(self, conv: &Convention) -> (Self, TransformOutcome) {
        let original = self.document.nodes;
        let (nodes, changed) = transform_nodes(original, conv, false, |node, skip| {
            let skip = skip || conv.is_marker_kind(&node.name);
            flatten_node(node, skip)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no pure namespace chain"),
        )
    }

    pub fn expand_dot_chain(self, kind: &str, conv: &Convention) -> (Self, TransformOutcome) {
        let mut nodes = self.document.nodes;
        let result = expand_dot_chain_nodes(&mut nodes, kind, conv);
        match result {
            Ok(changed) => (
                Self::from_kdl(KdlDocument { nodes }),
                outcome(changed, "no dot chain"),
            ),
            Err(reason) => (
                Self::from_kdl(KdlDocument { nodes }),
                TransformOutcome::NoOp { reason },
            ),
        }
    }

    pub fn collapse_dot_chain(self, kind: &str, conv: &Convention) -> (Self, TransformOutcome) {
        let original = self.document.nodes;
        match collapse_dot_chain_nodes(original.clone(), kind, conv) {
            Ok((nodes, changed)) => (
                Self::from_kdl(KdlDocument { nodes }),
                outcome(changed, "no nested chain"),
            ),
            Err(reason) => (
                Self::from_kdl(KdlDocument { nodes: original }),
                TransformOutcome::NoOp { reason },
            ),
        }
    }

    pub fn expand_properties(self, conv: &Convention) -> (Self, TransformOutcome) {
        let (nodes, changed) =
            transform_nodes(self.document.nodes, conv, false, expand_properties_node);
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no properties"),
        )
    }

    pub fn collapse_properties(self, conv: &Convention) -> (Self, TransformOutcome) {
        let original = self.document.nodes;
        match collapse_properties_nodes(original.clone(), conv, false) {
            Ok((nodes, changed)) => (
                Self::from_kdl(KdlDocument { nodes }),
                outcome(changed, "no property children"),
            ),
            Err(reason) => (
                Self::from_kdl(KdlDocument { nodes: original }),
                TransformOutcome::NoOp { reason },
            ),
        }
    }

    pub fn expand_type_annotations(
        self,
        marker: &Marker,
        conv: &Convention,
    ) -> (Self, TransformOutcome) {
        let (nodes, changed) = transform_nodes(self.document.nodes, conv, false, |node, skip| {
            expand_type_node(node, skip, marker)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no type annotations"),
        )
    }

    pub fn collapse_type_annotations(
        self,
        source: &Marker,
        conv: &Convention,
    ) -> (Self, TransformOutcome) {
        let (nodes, changed) = transform_nodes(self.document.nodes, conv, false, |node, skip| {
            collapse_type_node(node, skip, source)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no type marker"),
        )
    }

    pub fn expand_kind_keyword(
        self,
        marker: &Marker,
        conv: &Convention,
    ) -> (Self, TransformOutcome) {
        let (nodes, changed) = transform_nodes(self.document.nodes, conv, false, |node, skip| {
            expand_kind_node(node, skip, marker)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no kind keyword"),
        )
    }

    pub fn collapse_kind_keyword(
        self,
        source: &Marker,
        conv: &Convention,
    ) -> (Self, TransformOutcome) {
        let (nodes, changed) = transform_nodes(self.document.nodes, conv, false, |node, skip| {
            collapse_kind_node(node, skip, source)
        });
        (
            Self::from_kdl(KdlDocument { nodes }),
            outcome(changed, "no identifier marker"),
        )
    }
}

fn outcome(changed: bool, reason: &str) -> TransformOutcome {
    if changed {
        TransformOutcome::Applied
    } else {
        TransformOutcome::NoOp {
            reason: reason.to_owned(),
        }
    }
}

fn transform_nodes<F>(
    nodes: Vec<KdlNode>,
    conv: &Convention,
    parent_is_marker: bool,
    mut f: F,
) -> (Vec<KdlNode>, bool)
where
    F: FnMut(KdlNode, bool) -> (Vec<KdlNode>, bool) + Copy,
{
    let mut changed = false;
    let mut out = Vec::new();
    for mut node in nodes {
        if let Some(children) = node.children.take() {
            let skip_child_names = conv.is_marker_kind(&node.name);
            let (children, child_changed) = transform_nodes(children, conv, skip_child_names, f);
            node.children = (!children.is_empty()).then_some(children);
            changed |= child_changed;
        }
        let (mut replaced, node_changed) = f(node, parent_is_marker);
        out.append(&mut replaced);
        changed |= node_changed;
    }
    (out, changed)
}

fn nest_node(node: KdlNode, skip_name: bool) -> (Vec<KdlNode>, bool) {
    if skip_name || node.name.starts_with('.') || !node.name.contains('.') {
        return (vec![node], false);
    }
    let segments: Vec<_> = node.name.split('.').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return (vec![node], false);
    }
    let mut leaf = KdlNode {
        ty: node.ty,
        name: segments[segments.len() - 1].to_owned(),
        entries: node.entries,
        children: node.children,
    };
    for segment in segments[..segments.len() - 1].iter().rev() {
        leaf = KdlNode {
            ty: None,
            name: (*segment).to_owned(),
            entries: Vec::new(),
            children: Some(vec![leaf]),
        };
    }
    (vec![leaf], true)
}

fn flatten_node(mut node: KdlNode, skip_name: bool) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    let mut changed = false;
    while let Some(children) = node.children.take() {
        if node.ty.is_none() && node.entries.is_empty() && children.len() == 1 {
            let mut child = children.into_iter().next().expect("one child");
            child.name = format!("{}.{}", node.name, child.name);
            node = child;
            changed = true;
        } else {
            node.children = Some(children);
            break;
        }
    }
    (vec![node], changed)
}

fn expand_properties_node(mut node: KdlNode, skip_name: bool) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    let mut changed = false;
    let mut entries = Vec::new();
    let mut new_children = node.children.take().unwrap_or_default();
    for entry in node.entries {
        match entry {
            KdlEntry::Property { key, ty, value } => {
                new_children.push(KdlNode {
                    ty: None,
                    name: key,
                    entries: vec![KdlEntry::Argument { ty, value }],
                    children: None,
                });
                changed = true;
            }
            other => entries.push(other),
        }
    }
    node.entries = entries;
    node.children = (!new_children.is_empty()).then_some(new_children);
    (vec![node], changed)
}

fn collapse_properties_nodes(
    nodes: Vec<KdlNode>,
    conv: &Convention,
    parent_is_marker: bool,
) -> Result<(Vec<KdlNode>, bool), String> {
    let mut changed = false;
    let mut out = Vec::new();
    for mut node in nodes {
        if let Some(children) = node.children.take() {
            let skip_child_names = conv.is_marker_kind(&node.name);
            let (children, child_changed) =
                collapse_properties_nodes(children, conv, skip_child_names)?;
            node.children = (!children.is_empty()).then_some(children);
            changed |= child_changed;
        }
        if !parent_is_marker {
            let (collapsed, node_changed) = collapse_property_children(node)?;
            node = collapsed;
            changed |= node_changed;
        }
        out.push(node);
    }
    Ok((out, changed))
}

fn collapse_property_children(mut node: KdlNode) -> Result<(KdlNode, bool), String> {
    let Some(children) = node.children.take() else {
        return Ok((node, false));
    };
    let mut changed = false;
    let mut remaining = Vec::new();
    for child in children {
        match property_from_child(&child) {
            Some(entry) => {
                node.entries.push(entry);
                changed = true;
            }
            None if child.entries.len() > 1 => {
                return Err(format!("child {} has multiple entries", child.name));
            }
            None => remaining.push(child),
        }
    }
    node.children = (!remaining.is_empty()).then_some(remaining);
    Ok((node, changed))
}

fn property_from_child(child: &KdlNode) -> Option<KdlEntry> {
    if child.ty.is_some() || child.children.is_some() || child.entries.len() != 1 {
        return None;
    }
    match &child.entries[0] {
        KdlEntry::Argument { ty, value } => Some(KdlEntry::Property {
            key: child.name.clone(),
            ty: ty.clone(),
            value: value.clone(),
        }),
        KdlEntry::Property { .. } => None,
    }
}

fn expand_type_node(mut node: KdlNode, skip_name: bool, marker: &Marker) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    let Some(type_name) = node.ty.take() else {
        return (vec![node], false);
    };
    let wrapper = marker_wrapper(marker, &type_name, node);
    (vec![wrapper], true)
}

fn collapse_type_node(node: KdlNode, skip_name: bool, source: &Marker) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    match unwrap_marker(source, node.clone()) {
        Some((type_name, mut child)) => {
            child.ty = Some(type_name);
            (vec![child], true)
        }
        None => (vec![node], false),
    }
}

fn expand_kind_node(mut node: KdlNode, skip_name: bool, marker: &Marker) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    let entries = std::mem::take(&mut node.entries);
    let Some((identifier, rest_entries)) = take_first_string_arg(entries) else {
        return (vec![node], false);
    };
    let marker = KdlNode {
        ty: None,
        name: marker_name(marker, &identifier),
        entries: Vec::new(),
        children: node.children.take(),
    };
    let mut children = vec![marker];
    node.entries = rest_entries;
    node.children = Some(std::mem::take(&mut children));
    (vec![node], true)
}

fn collapse_kind_node(mut node: KdlNode, skip_name: bool, source: &Marker) -> (Vec<KdlNode>, bool) {
    if skip_name {
        return (vec![node], false);
    }
    let Some(mut children) = node.children.take() else {
        return (vec![node], false);
    };
    if children.len() != 1 {
        node.children = Some(children);
        return (vec![node], false);
    }
    let marker = children.remove(0);
    if !marker_matches(source, &marker.name) {
        node.children = Some(vec![marker]);
        return (vec![node], false);
    }
    let identifier = marker_payload(source, &marker);
    node.entries.insert(
        0,
        KdlEntry::Argument {
            ty: None,
            value: KdlValue::String(identifier),
        },
    );
    node.children = marker.children;
    (vec![node], true)
}

fn take_first_string_arg(entries: Vec<KdlEntry>) -> Option<(String, Vec<KdlEntry>)> {
    let mut found = None;
    let mut rest = Vec::new();
    for entry in entries {
        match (found.is_none(), entry) {
            (
                true,
                KdlEntry::Argument {
                    ty: None,
                    value: KdlValue::String(value),
                },
            ) => found = Some(value),
            (_, entry) => rest.push(entry),
        }
    }
    found.map(|value| (value, rest))
}

fn marker_wrapper(marker: &Marker, payload: &str, child: KdlNode) -> KdlNode {
    match marker {
        Marker::Kind(kind) => KdlNode {
            ty: None,
            name: kind.clone(),
            entries: vec![KdlEntry::Argument {
                ty: None,
                value: KdlValue::String(payload.to_owned()),
            }],
            children: Some(vec![child]),
        },
        Marker::Prefix(prefix) => KdlNode {
            ty: None,
            name: format!("{prefix}{payload}"),
            entries: Vec::new(),
            children: Some(vec![child]),
        },
    }
}

fn unwrap_marker(marker: &Marker, node: KdlNode) -> Option<(String, KdlNode)> {
    if !marker_matches(marker, &node.name) {
        return None;
    }
    let payload = marker_payload(marker, &node);
    let mut children = node.children?;
    (children.len() == 1).then(|| (payload, children.remove(0)))
}

fn marker_matches(marker: &Marker, name: &str) -> bool {
    match marker {
        Marker::Kind(kind) => name == kind,
        Marker::Prefix(prefix) => name.starts_with(*prefix),
    }
}

fn marker_payload(marker: &Marker, node: &KdlNode) -> String {
    match marker {
        Marker::Kind(_) => node.first_string_arg().unwrap_or_default().to_owned(),
        Marker::Prefix(prefix) => node
            .name
            .strip_prefix(*prefix)
            .unwrap_or(&node.name)
            .to_owned(),
    }
}

fn marker_name(marker: &Marker, payload: &str) -> String {
    match marker {
        Marker::Kind(kind) => kind.clone(),
        Marker::Prefix(prefix) => format!("{prefix}{payload}"),
    }
}

fn expand_dot_chain_nodes(
    nodes: &mut Vec<KdlNode>,
    kind: &str,
    conv: &Convention,
) -> Result<bool, String> {
    let mut changed = false;
    for node in nodes.iter_mut() {
        if let Some(children) = node.children.as_mut() {
            if conv.is_marker_kind(&node.name) {
                continue;
            }
            changed |= expand_dot_chain_nodes(children, kind, conv)?;
        }
    }
    let original = std::mem::take(nodes);
    let mut out = Vec::new();
    let mut stack: Vec<Vec<usize>> = Vec::new();
    for mut node in original {
        let depth = node.name.chars().take_while(|c| *c == '.').count();
        let node_kind = node.name.trim_start_matches('.');
        if node_kind != kind {
            out.push(node);
            stack.clear();
            continue;
        }
        if depth > stack.len() {
            return Err(format!("depth jump from {} to {}", stack.len(), depth));
        }
        if depth > 0 {
            changed = true;
        }
        node.name = node_kind.to_owned();
        if depth == 0 {
            out.push(node);
            stack.clear();
            stack.push(vec![out.len() - 1]);
        } else {
            let parent_path = stack[depth - 1].clone();
            let parent = get_mut_at_path(&mut out, &parent_path);
            let children = parent.children.get_or_insert_with(Vec::new);
            children.push(node);
            let mut new_path = parent_path;
            new_path.push(children.len() - 1);
            stack.truncate(depth);
            stack.push(new_path);
        }
    }
    *nodes = out;
    Ok(changed)
}

fn get_mut_at_path<'a>(nodes: &'a mut [KdlNode], path: &[usize]) -> &'a mut KdlNode {
    let (first, rest) = path.split_first().expect("non-empty path");
    let mut current = &mut nodes[*first];
    for index in rest {
        current = &mut current.children.as_mut().expect("path children exist")[*index];
    }
    current
}

fn collapse_dot_chain_nodes(
    nodes: Vec<KdlNode>,
    kind: &str,
    conv: &Convention,
) -> Result<(Vec<KdlNode>, bool), String> {
    let mut out = Vec::new();
    let mut changed = false;
    for mut node in nodes {
        if node.name == kind {
            collect_chain(node, kind, 0, &mut out)?;
            changed = true;
            continue;
        }
        if let Some(children) = node.children.take() {
            if conv.is_marker_kind(&node.name) {
                node.children = Some(children);
            } else {
                let (children, child_changed) = collapse_dot_chain_nodes(children, kind, conv)?;
                node.children = (!children.is_empty()).then_some(children);
                changed |= child_changed;
            }
        }
        out.push(node);
    }
    Ok((out, changed))
}

fn collect_chain(
    mut node: KdlNode,
    kind: &str,
    depth: usize,
    out: &mut Vec<KdlNode>,
) -> Result<(), String> {
    let children = node.children.take().unwrap_or_default();
    let mut chain_children = Vec::new();
    for child in children {
        if child.name != kind {
            return Err(format!("mixed child kind under {}", node.name));
        }
        chain_children.push(child);
    }
    if depth > 0 {
        node.name = format!("{}{}", ".".repeat(depth), node.name);
    }
    out.push(node);
    for child in chain_children {
        collect_chain(child, kind, depth + 1, out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TransformOutcome;
    use crate::{Convention, Document, Marker};

    fn names(doc: &Document) -> Vec<String> {
        doc.as_kdl()
            .nodes()
            .iter()
            .map(|n| n.name.clone())
            .collect()
    }

    #[test]
    fn flatten_pure_namespace_chain() {
        let doc = Document::parse("a { b { c } }\n").expect("parse");
        let (doc, outcome) = doc.flatten(&Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert_eq!(names(&doc), vec!["a.b.c"]);
    }

    #[test]
    fn flatten_with_args_at_intermediate_no_op() {
        let doc = Document::parse("a \"x\" { b }\n").expect("parse");
        let (_, outcome) = doc.flatten(&Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn flatten_with_property_at_intermediate_no_op() {
        let doc = Document::parse("a key=\"x\" { b }\n").expect("parse");
        let (_, outcome) = doc.flatten(&Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn flatten_with_siblings_no_op() {
        let doc = Document::parse("a { b\n c }\n").expect("parse");
        let (_, outcome) = doc.flatten(&Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn nest_dot_collapsed_namespace() {
        let doc = Document::parse("a.b.c\n").expect("parse");
        let (doc, outcome) = doc.nest(&Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert_eq!(doc.as_kdl().nodes()[0].name, "a");
    }

    #[test]
    fn flatten_nest_round_trip() {
        let doc = Document::parse("a.b.c\n").expect("parse");
        let (nested, _) = doc.clone().nest(&Convention::new());
        let (flat, _) = nested.flatten(&Convention::new());
        assert_eq!(flat.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_dot_chain_linear() {
        let doc = Document::parse("call \"a\"\n.call \"b\"\n..call \"c\"\n").expect("parse");
        let (doc, outcome) = doc.expand_dot_chain("call", &Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert!(doc.as_kdl().nodes()[0].children.is_some());
    }

    #[test]
    fn expand_dot_chain_branching() {
        let doc = Document::parse("call \"a\"\n.call \"b1\"\n.call \"b2\"\n").expect("parse");
        let (doc, _) = doc.expand_dot_chain("call", &Convention::new());
        assert_eq!(
            doc.as_kdl().nodes()[0]
                .children
                .as_ref()
                .expect("children")
                .len(),
            2
        );
    }

    #[test]
    fn expand_dot_chain_three_depth() {
        let doc = Document::parse("call \"a\"\n.call \"b\"\n..call \"c\"\n").expect("parse");
        let (doc, _) = doc.expand_dot_chain("call", &Convention::new());
        let child = &doc.as_kdl().nodes()[0].children.as_ref().expect("child")[0];
        assert!(child.children.is_some());
    }

    #[test]
    fn expand_dot_chain_kind_filter_skips_others() {
        let doc = Document::parse("item \"a\"\n.item \"b\"\n").expect("parse");
        let (_, outcome) = doc.expand_dot_chain("call", &Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn expand_dot_chain_depth_jump_no_op() {
        let doc = Document::parse("call \"a\"\n..call \"b\"\n").expect("parse");
        let (_, outcome) = doc.expand_dot_chain("call", &Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn collapse_dot_chain_linear() {
        let doc = Document::parse("call \"a\" { call \"b\" { call \"c\" } }\n").expect("parse");
        let (doc, outcome) = doc.collapse_dot_chain("call", &Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert_eq!(names(&doc), vec!["call", ".call", "..call"]);
    }

    #[test]
    fn collapse_dot_chain_with_mixed_kind_no_op() {
        let doc = Document::parse("call \"a\" { item \"b\" }\n").expect("parse");
        let (_, outcome) = doc.collapse_dot_chain("call", &Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn expand_collapse_round_trip_linear() {
        let doc = Document::parse("call \"a\"\n.call \"b\"\n..call \"c\"\n").expect("parse");
        let (expanded, _) = doc.clone().expand_dot_chain("call", &Convention::new());
        let (collapsed, _) = expanded.collapse_dot_chain("call", &Convention::new());
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_collapse_round_trip_branching() {
        let doc = Document::parse("call \"a\"\n.call \"b1\"\n.call \"b2\"\n").expect("parse");
        let (expanded, _) = doc.clone().expand_dot_chain("call", &Convention::new());
        let (collapsed, _) = expanded.collapse_dot_chain("call", &Convention::new());
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_properties_basic() {
        let doc = Document::parse("node a=1 b=2\n").expect("parse");
        let (doc, outcome) = doc.expand_properties(&Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert_eq!(
            doc.as_kdl().nodes()[0]
                .children
                .as_ref()
                .expect("children")
                .len(),
            2
        );
    }

    #[test]
    fn expand_properties_with_type_annotation() {
        let doc = Document::parse("node val=(i32)42\n").expect("parse");
        let (doc, _) = doc.expand_properties(&Convention::new());
        let child = &doc.as_kdl().nodes()[0].children.as_ref().expect("children")[0];
        assert_eq!(child.entries[0].ty(), Some("i32"));
    }

    #[test]
    fn expand_properties_under_marker_no_op() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        let doc = Document::parse(":m { node a=1 }\n").expect("parse");
        let (_, outcome) = doc.expand_properties(&conv);
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn collapse_properties_basic() {
        let doc = Document::parse("node { a 1\n b 2 }\n").expect("parse");
        let (doc, outcome) = doc.collapse_properties(&Convention::new());
        assert_eq!(outcome, TransformOutcome::Applied);
        assert_eq!(doc.as_kdl().nodes()[0].entries.len(), 2);
    }

    #[test]
    fn collapse_properties_with_multi_arg_no_op() {
        let doc = Document::parse("node { a 1 2 }\n").expect("parse");
        let (_, outcome) = doc.collapse_properties(&Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn collapse_properties_with_children_no_op() {
        let doc = Document::parse("node { a 1 { b } }\n").expect("parse");
        let (_, outcome) = doc.collapse_properties(&Convention::new());
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn expand_collapse_properties_round_trip() {
        let doc = Document::parse("node a=1 b=(i32)2\n").expect("parse");
        let (expanded, _) = doc.clone().expand_properties(&Convention::new());
        let (collapsed, _) = expanded.collapse_properties(&Convention::new());
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_type_annotations_kind_marker() {
        let doc = Document::parse("(integer)value\n").expect("parse");
        let (doc, _) = doc.expand_type_annotations(
            &Marker::Kind("_type".to_owned()),
            &Convention::new().with_marker(Marker::Kind("_type".to_owned())),
        );
        assert_eq!(doc.as_kdl().nodes()[0].name, "_type");
    }

    #[test]
    fn expand_type_annotations_prefix_marker() {
        let doc = Document::parse("(integer)value\n").expect("parse");
        let (doc, _) = doc.expand_type_annotations(
            &Marker::Prefix(':'),
            &Convention::new().with_marker(Marker::Prefix(':')),
        );
        assert_eq!(doc.as_kdl().nodes()[0].name, ":integer");
    }

    #[test]
    fn expand_type_annotations_under_existing_marker_no_op() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        let doc = Document::parse(":m { (integer)value }\n").expect("parse");
        let (_, outcome) = doc.expand_type_annotations(&Marker::Prefix(':'), &conv);
        assert!(matches!(outcome, TransformOutcome::NoOp { .. }));
    }

    #[test]
    fn collapse_type_annotations_kind_marker() {
        let doc = Document::parse("_type \"integer\" { value }\n").expect("parse");
        let (doc, _) = doc.collapse_type_annotations(
            &Marker::Kind("_type".to_owned()),
            &Convention::new().with_marker(Marker::Kind("_type".to_owned())),
        );
        assert_eq!(doc.as_kdl().nodes()[0].ty(), Some("integer"));
    }

    #[test]
    fn collapse_type_annotations_prefix_marker() {
        let doc = Document::parse(":integer { value }\n").expect("parse");
        let (doc, _) = doc.collapse_type_annotations(
            &Marker::Prefix(':'),
            &Convention::new().with_marker(Marker::Prefix(':')),
        );
        assert_eq!(doc.as_kdl().nodes()[0].ty(), Some("integer"));
    }

    #[test]
    fn expand_collapse_type_annotations_round_trip_kind() {
        let conv = Convention::new().with_marker(Marker::Kind("_type".to_owned()));
        let marker = Marker::Kind("_type".to_owned());
        let doc = Document::parse("(integer)value\n").expect("parse");
        let (expanded, _) = doc.clone().expand_type_annotations(&marker, &conv);
        let (collapsed, _) = expanded.collapse_type_annotations(&marker, &conv);
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_collapse_type_annotations_round_trip_prefix() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        let marker = Marker::Prefix(':');
        let doc = Document::parse("(integer)value\n").expect("parse");
        let (expanded, _) = doc.clone().expand_type_annotations(&marker, &conv);
        let (collapsed, _) = expanded.collapse_type_annotations(&marker, &conv);
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn expand_kind_keyword_prefix_marker() {
        let doc = Document::parse("entity \"alpha\" { body }\n").expect("parse");
        let (doc, _) = doc.expand_kind_keyword(&Marker::Prefix('@'), &Convention::new());
        let child = &doc.as_kdl().nodes()[0].children.as_ref().expect("children")[0];
        assert_eq!(child.name, "@alpha");
    }

    #[test]
    fn expand_kind_keyword_kind_marker() {
        let doc = Document::parse("entity \"alpha\" { body }\n").expect("parse");
        let (doc, _) = doc.expand_kind_keyword(&Marker::Kind("_id".to_owned()), &Convention::new());
        let child = &doc.as_kdl().nodes()[0].children.as_ref().expect("children")[0];
        assert_eq!(child.name, "_id");
    }

    #[test]
    fn collapse_kind_keyword_prefix_marker() {
        let doc = Document::parse("entity { @alpha { body } }\n").expect("parse");
        let (doc, _) = doc.collapse_kind_keyword(&Marker::Prefix('@'), &Convention::new());
        assert_eq!(doc.as_kdl().nodes()[0].first_string_arg(), Some("alpha"));
    }

    #[test]
    fn collapse_kind_keyword_kind_marker() {
        let doc = Document::parse("entity { _id \"alpha\" { body } }\n").expect("parse");
        let (doc, _) =
            doc.collapse_kind_keyword(&Marker::Kind("_id".to_owned()), &Convention::new());
        assert_eq!(doc.as_kdl().nodes()[0].first_string_arg(), Some("alpha"));
    }

    #[test]
    fn expand_collapse_kind_keyword_round_trip() {
        let doc = Document::parse("entity \"alpha\" { body }\n").expect("parse");
        let marker = Marker::Prefix('@');
        let (expanded, _) = doc.clone().expand_kind_keyword(&marker, &Convention::new());
        let (collapsed, _) = expanded.collapse_kind_keyword(&marker, &Convention::new());
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }

    #[test]
    fn reverse_domain_identifier_round_trip() {
        let conv = Convention::new().with_marker(Marker::Prefix(':'));
        let marker = Marker::Prefix(':');
        let doc = Document::parse("(unit)com.vscodium.codium-insiders\n").expect("parse");
        let (expanded, _) = doc.clone().expand_type_annotations(&marker, &conv);
        let (nested, _) = expanded.nest(&conv);
        let (flat, _) = nested.flatten(&conv);
        let (collapsed, _) = flat.collapse_type_annotations(&marker, &conv);
        assert_eq!(collapsed.as_kdl(), doc.as_kdl());
    }
}
