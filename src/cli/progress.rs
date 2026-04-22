use crate::cli::table::{build_profile, format_from_flags, retry_policy, unwrap_or_raw};
use crate::cli::{GlobalFlags, ProgressArgs};
use crate::client::Client;
use crate::error::{Error, Result};
use crate::output::emit_value;
use std::io;

pub fn run(global: &GlobalFlags, args: ProgressArgs) -> Result<()> {
    let profile = build_profile(global)?;
    let client = Client::builder()
        .retry(retry_policy(global.no_retry))
        .build(&profile)?;
    let path = format!("/api/sn_cicd/progress/{}", args.progress_id);
    let resp = client.get(&path, &[])?;
    let out = unwrap_or_raw(resp, global.output);
    emit_value(io::stdout().lock(), &out, format_from_flags(global))
        .map_err(|e| Error::Usage(format!("stdout: {e}")))
}
