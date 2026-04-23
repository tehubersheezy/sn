#![allow(dead_code)]

pub fn mock_profile(instance: &str) -> sn::config::ResolvedProfile {
    sn::config::ResolvedProfile {
        name: "test".into(),
        instance: instance.to_string(),
        username: "admin".into(),
        password: "pw".into(),
        proxy: None,
        no_proxy: None,
        insecure: false,
        ca_cert: None,
        proxy_ca_cert: None,
        proxy_username: None,
        proxy_password: None,
    }
}
