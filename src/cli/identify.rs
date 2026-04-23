use crate::body::{build_body, BodyInput};
use crate::cli::table::{build_client, build_profile, format_from_flags, unwrap_or_raw};
use crate::cli::{GlobalFlags, IdentifyArgs, IdentifyEnhancedArgs};
use crate::error::{Error, Result};
use crate::output::emit_value;
use std::io;

pub fn create_update(global: &GlobalFlags, args: IdentifyArgs) -> Result<()> {
    let profile = build_profile(global)?;
    let client = build_client(&profile, global.timeout)?;
    let body_input = if let Some(d) = args.data {
        BodyInput::Data(d)
    } else if !args.field.is_empty() {
        BodyInput::Fields(args.field)
    } else {
        BodyInput::None
    };
    let body = build_body(body_input)?;
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(v) = args.data_source {
        query.push(("sysparm_data_source".into(), v));
    }
    let resp = client.post("/api/now/identifyreconcile", &query, &body)?;
    let out = unwrap_or_raw(resp, global.output);
    emit_value(io::stdout().lock(), &out, format_from_flags(global))
        .map_err(|e| Error::Usage(format!("stdout: {e}")))
}

pub fn query(global: &GlobalFlags, args: IdentifyArgs) -> Result<()> {
    let profile = build_profile(global)?;
    let client = build_client(&profile, global.timeout)?;
    let body_input = if let Some(d) = args.data {
        BodyInput::Data(d)
    } else if !args.field.is_empty() {
        BodyInput::Fields(args.field)
    } else {
        BodyInput::None
    };
    let body = build_body(body_input)?;
    let mut query_params: Vec<(String, String)> = Vec::new();
    if let Some(v) = args.data_source {
        query_params.push(("sysparm_data_source".into(), v));
    }
    let resp = client.post("/api/now/identifyreconcile/query", &query_params, &body)?;
    let out = unwrap_or_raw(resp, global.output);
    emit_value(io::stdout().lock(), &out, format_from_flags(global))
        .map_err(|e| Error::Usage(format!("stdout: {e}")))
}

pub fn create_update_enhanced(global: &GlobalFlags, args: IdentifyEnhancedArgs) -> Result<()> {
    let profile = build_profile(global)?;
    let client = build_client(&profile, global.timeout)?;
    let body_input = if let Some(d) = args.data {
        BodyInput::Data(d)
    } else if !args.field.is_empty() {
        BodyInput::Fields(args.field)
    } else {
        BodyInput::None
    };
    let body = build_body(body_input)?;
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(v) = args.data_source {
        query.push(("sysparm_data_source".into(), v));
    }
    if let Some(v) = args.options {
        query.push(("options".into(), v));
    }
    let resp = client.post("/api/now/identifyreconcile/enhanced", &query, &body)?;
    let out = unwrap_or_raw(resp, global.output);
    emit_value(io::stdout().lock(), &out, format_from_flags(global))
        .map_err(|e| Error::Usage(format!("stdout: {e}")))
}

pub fn query_enhanced(global: &GlobalFlags, args: IdentifyEnhancedArgs) -> Result<()> {
    let profile = build_profile(global)?;
    let client = build_client(&profile, global.timeout)?;
    let body_input = if let Some(d) = args.data {
        BodyInput::Data(d)
    } else if !args.field.is_empty() {
        BodyInput::Fields(args.field)
    } else {
        BodyInput::None
    };
    let body = build_body(body_input)?;
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(v) = args.data_source {
        query.push(("sysparm_data_source".into(), v));
    }
    if let Some(v) = args.options {
        query.push(("options".into(), v));
    }
    let resp = client.post("/api/now/identifyreconcile/queryEnhanced", &query, &body)?;
    let out = unwrap_or_raw(resp, global.output);
    emit_value(io::stdout().lock(), &out, format_from_flags(global))
        .map_err(|e| Error::Usage(format!("stdout: {e}")))
}
