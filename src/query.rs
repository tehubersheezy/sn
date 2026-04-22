use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayValue {
    True,
    False,
    All,
}

impl DisplayValue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ListQuery {
    pub query: Option<String>,
    pub fields: Option<String>,
    pub page_size: Option<u32>,
    pub offset: Option<u32>,
    pub display_value: Option<DisplayValue>,
    pub exclude_reference_link: Option<bool>,
    pub suppress_pagination_header: Option<bool>,
    pub view: Option<String>,
    pub query_category: Option<String>,
    pub query_no_domain: Option<bool>,
    pub no_count: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct GetQuery {
    pub fields: Option<String>,
    pub display_value: Option<DisplayValue>,
    pub exclude_reference_link: Option<bool>,
    pub view: Option<String>,
    pub query_no_domain: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct WriteQuery {
    pub fields: Option<String>,
    pub display_value: Option<DisplayValue>,
    pub exclude_reference_link: Option<bool>,
    pub input_display_value: Option<bool>,
    pub suppress_auto_sys_field: Option<bool>,
    pub view: Option<String>,
    pub query_no_domain: Option<bool>, // PATCH/PUT only; POST ignores
}

#[derive(Debug, Default, Clone)]
pub struct DeleteQuery {
    pub query_no_domain: Option<bool>,
}

fn push(pairs: &mut Vec<(String, String)>, key: &str, val: Option<String>) {
    if let Some(v) = val {
        pairs.push((key.into(), v));
    }
}

fn push_bool(pairs: &mut Vec<(String, String)>, key: &str, val: Option<bool>) {
    if let Some(v) = val {
        pairs.push((key.into(), v.to_string()));
    }
}

fn push_u32(pairs: &mut Vec<(String, String)>, key: &str, val: Option<u32>) {
    if let Some(v) = val {
        pairs.push((key.into(), v.to_string()));
    }
}

impl ListQuery {
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        push(&mut p, "sysparm_query", self.query.clone());
        push(&mut p, "sysparm_fields", self.fields.clone());
        push_u32(&mut p, "sysparm_limit", self.page_size);
        push_u32(&mut p, "sysparm_offset", self.offset);
        push(
            &mut p,
            "sysparm_display_value",
            self.display_value.map(|d| d.as_str().to_string()),
        );
        push_bool(
            &mut p,
            "sysparm_exclude_reference_link",
            self.exclude_reference_link,
        );
        push_bool(
            &mut p,
            "sysparm_suppress_pagination_header",
            self.suppress_pagination_header,
        );
        push(&mut p, "sysparm_view", self.view.clone());
        push(
            &mut p,
            "sysparm_query_category",
            self.query_category.clone(),
        );
        push_bool(&mut p, "sysparm_query_no_domain", self.query_no_domain);
        push_bool(&mut p, "sysparm_no_count", self.no_count);
        p
    }
}

impl GetQuery {
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        push(&mut p, "sysparm_fields", self.fields.clone());
        push(
            &mut p,
            "sysparm_display_value",
            self.display_value.map(|d| d.as_str().to_string()),
        );
        push_bool(
            &mut p,
            "sysparm_exclude_reference_link",
            self.exclude_reference_link,
        );
        push(&mut p, "sysparm_view", self.view.clone());
        push_bool(&mut p, "sysparm_query_no_domain", self.query_no_domain);
        p
    }
}

impl WriteQuery {
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        push(&mut p, "sysparm_fields", self.fields.clone());
        push(
            &mut p,
            "sysparm_display_value",
            self.display_value.map(|d| d.as_str().to_string()),
        );
        push_bool(
            &mut p,
            "sysparm_exclude_reference_link",
            self.exclude_reference_link,
        );
        push_bool(
            &mut p,
            "sysparm_input_display_value",
            self.input_display_value,
        );
        push_bool(
            &mut p,
            "sysparm_suppress_auto_sys_field",
            self.suppress_auto_sys_field,
        );
        push(&mut p, "sysparm_view", self.view.clone());
        push_bool(&mut p, "sysparm_query_no_domain", self.query_no_domain);
        p
    }
}

impl DeleteQuery {
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        push_bool(&mut p, "sysparm_query_no_domain", self.query_no_domain);
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_query_emits_only_set_pairs() {
        let q = ListQuery {
            query: Some("active=true".into()),
            page_size: Some(10),
            ..Default::default()
        };
        let pairs = q.to_pairs();
        assert_eq!(
            pairs,
            vec![
                ("sysparm_query".into(), "active=true".into()),
                ("sysparm_limit".into(), "10".into()),
            ]
        );
    }

    #[test]
    fn display_value_serialises_as_lowercase_string() {
        let q = ListQuery {
            display_value: Some(DisplayValue::All),
            ..Default::default()
        };
        assert_eq!(
            q.to_pairs(),
            vec![("sysparm_display_value".into(), "all".into())]
        );
    }

    #[test]
    fn write_query_respects_all_fields() {
        let q = WriteQuery {
            fields: Some("a,b".into()),
            input_display_value: Some(true),
            suppress_auto_sys_field: Some(true),
            display_value: Some(DisplayValue::False),
            ..Default::default()
        };
        let pairs = q.to_pairs();
        assert!(pairs.contains(&("sysparm_fields".into(), "a,b".into())));
        assert!(pairs.contains(&("sysparm_input_display_value".into(), "true".into())));
        assert!(pairs.contains(&("sysparm_suppress_auto_sys_field".into(), "true".into())));
        assert!(pairs.contains(&("sysparm_display_value".into(), "false".into())));
    }

    #[test]
    fn empty_query_emits_no_pairs() {
        let q = ListQuery::default();
        assert!(q.to_pairs().is_empty());
    }
}
