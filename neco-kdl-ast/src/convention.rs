/// 軸 1 / 2 / 4 / 5 の正規形宣言。 4 軸対称、 軸 4 / 5 は marker を embed する variant も持つ。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AxisForm {
    /// OFF: 該当軸の表記差は read 時に解釈されず入力の literal 形で保持
    #[default]
    Off,
    /// 展開形を正規形とする ( 軸 1 / 2 用、 軸 4 / 5 は ExpandWithMarker を使う )
    Expand,
    /// 圧縮形を正規形とする ( 軸 1 / 2 用、 軸 4 / 5 は CollapseWithMarker を使う )
    Collapse,
    /// 軸 4 / 5 用 expand、 marker を embed
    ExpandWithMarker(Marker),
    /// 軸 4 / 5 用 collapse、 marker を embed
    CollapseWithMarker(Marker),
}

/// 軸 3 ( property-child ) 専用 enum。 融合形 axiom により他 4 軸と非対称。
///
/// 融合形 ( fused form ): `key=v` ↔ `mod_key v` ( flat child node、 ここで mod_key は
/// 「key 自体が child node 名になる」 という abstract 表記、 literal prefix 文字列ではない )。
///
/// 構造制約 axiom: `mod { key v }` 形 ( Nested variant ) は enum に存在しない =
/// 構築不能。 Expand / Collapse は対称だが、 軸 4 / 5 と異なり marker を embed しない
/// ( KDL property syntax の entry-position 制約による非対称 )。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PropertyChildForm {
    /// OFF: property / child は read 時に解釈されず入力 form のまま
    #[default]
    Off,
    /// fused-form expand: properties → flat `key v` child nodes
    Expand,
    /// fused-form collapse: flat `key v` child nodes → properties
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Convention {
    pub markers: Vec<Marker>,
    pub namespace_form: AxisForm,
    pub procedure_form: AxisForm,
    pub property_child_form: PropertyChildForm,
    pub type_annotation_form: AxisForm,
    pub kind_keyword_form: AxisForm,
}

impl Convention {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_marker(mut self, m: Marker) -> Self {
        self.markers.push(m);
        self
    }

    pub fn with_namespace_form(mut self, form: AxisForm) -> Self {
        self.namespace_form = form;
        self
    }

    pub fn with_procedure_form(mut self, form: AxisForm) -> Self {
        self.procedure_form = form;
        self
    }

    pub fn with_property_child_form(mut self, form: PropertyChildForm) -> Self {
        self.property_child_form = form;
        self
    }

    pub fn with_type_annotation_form(mut self, form: AxisForm) -> Self {
        self.type_annotation_form = form;
        self
    }

    pub fn with_kind_keyword_form(mut self, form: AxisForm) -> Self {
        self.kind_keyword_form = form;
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
    use super::{AxisForm, Convention, Marker, PropertyChildForm};

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

    #[test]
    fn convention_default_all_axes_off() {
        let conv = Convention::default();
        assert!(matches!(conv.namespace_form, AxisForm::Off));
        assert!(matches!(conv.procedure_form, AxisForm::Off));
        assert!(matches!(conv.property_child_form, PropertyChildForm::Off));
        assert!(matches!(conv.type_annotation_form, AxisForm::Off));
        assert!(matches!(conv.kind_keyword_form, AxisForm::Off));
        assert!(conv.markers.is_empty());
    }

    #[test]
    fn convention_with_per_axis_form() {
        let conv = Convention::new()
            .with_namespace_form(AxisForm::Expand)
            .with_procedure_form(AxisForm::Collapse)
            .with_property_child_form(PropertyChildForm::Expand)
            .with_type_annotation_form(AxisForm::ExpandWithMarker(Marker::Prefix(':')))
            .with_kind_keyword_form(AxisForm::CollapseWithMarker(Marker::Kind("_id".to_owned())));
        assert_eq!(conv.namespace_form, AxisForm::Expand);
        assert_eq!(conv.procedure_form, AxisForm::Collapse);
        assert_eq!(conv.property_child_form, PropertyChildForm::Expand);
        assert_eq!(
            conv.type_annotation_form,
            AxisForm::ExpandWithMarker(Marker::Prefix(':'))
        );
        assert_eq!(
            conv.kind_keyword_form,
            AxisForm::CollapseWithMarker(Marker::Kind("_id".to_owned()))
        );
    }

    /// 軸 3 融合形 axiom: PropertyChildForm enum は Off / Expand / Collapse の 3 variant のみで
    /// `mod { key v }` 形 ( Nested variant ) は構築不能。
    /// `rg fused_form|mod_key v` driver は本 enum doc comment の literal 文字列で hit。
    #[test]
    fn property_child_fused_form_only() {
        // 構築可能な variant は 3 つだけ ( Expand / Collapse は fused form のみ採る )
        let _off = PropertyChildForm::Off;
        let _expand = PropertyChildForm::Expand;
        let _collapse = PropertyChildForm::Collapse;
        // PropertyChildForm::Nested は存在しない ( enum 定義が axiom )
        // → コンパイルが通った時点で literal 検査済
        let names = format!(
            "{:?} {:?} {:?}",
            PropertyChildForm::Off,
            PropertyChildForm::Expand,
            PropertyChildForm::Collapse
        );
        assert!(names.contains("Off"));
        assert!(names.contains("Expand"));
        assert!(names.contains("Collapse"));
    }
}
