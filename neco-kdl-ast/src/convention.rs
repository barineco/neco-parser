/// 軸 1 / 2 / 4 / 5 の正規形宣言。 4 軸は対称であり、 軸 4 / 5 は marker を内包する variant も持つ。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AxisForm {
    /// OFF: 該当軸の表記差を読み取り時に解釈せず、 入力のまま保持する
    #[default]
    Off,
    /// 展開形を正規形とする ( 軸 1 / 2 用。 軸 4 / 5 は marker 内包 variant を使う )
    Expand,
    /// 圧縮形を正規形とする ( 軸 1 / 2 用。 軸 4 / 5 は marker 内包 variant を使う )
    Collapse,
    /// 軸 4 / 5 用の展開形、 marker を内包する
    ExpandWithMarker(Marker),
    /// 軸 4 / 5 用の圧縮形、 marker を内包する
    CollapseWithMarker(Marker),
}

/// 軸 3 ( property-child ) 専用の enum。 融合形の性質により他 4 軸と非対称である。
/// property と子 node の相互変換に variant を限定し、 他軸と異なり marker を
/// 内包しない構成を採る。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PropertyChildForm {
    /// OFF: property と child は読み取り時に解釈せず、 入力の形のまま保持する
    #[default]
    Off,
    /// 展開: property を key-value の子 node へ変換する
    Expand,
    /// 圧縮: key-value の子 node を property へ変換する
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

    #[test]
    fn property_child_fused_form_only() {
        let _off = PropertyChildForm::Off;
        let _expand = PropertyChildForm::Expand;
        let _collapse = PropertyChildForm::Collapse;

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
