use crate::cli::GlobalFlags;
use crate::client::{Client, RetryPolicy};
use crate::config::{
    config_path, credentials_path, load_config_from, load_credentials_from, resolve_profile,
    ProfileResolverInputs,
};
use crate::error::Result;
use serde_json::json;
use std::io::Write;

pub fn test(global: &GlobalFlags) -> Result<()> {
    let config = load_config_from(&config_path()?)?;
    let creds = load_credentials_from(&credentials_path()?)?;

    let env_profile = std::env::var("SN_PROFILE").ok();
    let env_instance = std::env::var("SN_INSTANCE").ok();
    let env_username = std::env::var("SN_USERNAME").ok();
    let env_password = std::env::var("SN_PASSWORD").ok();

    let profile = resolve_profile(ProfileResolverInputs {
        cli_profile: global.profile.as_deref(),
        env_profile: env_profile.as_deref(),
        cli_instance_override: global.instance_override.as_deref(),
        env_instance: env_instance.as_deref(),
        env_username: env_username.as_deref(),
        env_password: env_password.as_deref(),
        config: &config,
        credentials: &creds,
    })?;

    let retry = if global.no_retry {
        RetryPolicy {
            enabled: false,
            ..Default::default()
        }
    } else {
        RetryPolicy::default()
    };
    let client = Client::builder().retry(retry).build(&profile)?;
    let v = client.get(
        "/api/now/table/sys_user",
        &[("sysparm_limit".into(), "1".into())],
    )?;
    let user = v["result"]
        .get(0)
        .and_then(|r| r.get("user_name"))
        .and_then(|x| x.as_str())
        .unwrap_or(&profile.username);
    let msg = json!({
        "ok": true,
        "instance": profile.instance,
        "username": user,
        "profile": profile.name,
    });
    writeln!(std::io::stderr(), "{msg}").ok();
    Ok(())
}
