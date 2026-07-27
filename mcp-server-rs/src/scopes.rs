//! Tool scope definitions for gateway access control.
//!
//! Each tool is tagged with one or more scopes. When WM_SCOPES is set,
//! only tools matching the configured scopes are exposed.

/// Returns the scope(s) for a given tool name.
pub fn tool_scope(name: &str) -> &'static [&'static str] {
    match name {
        // ── admin ───────────────────────────────────────────────
        "is_status" | "is_shutdown" | "list_instances" => &["admin", "monitor"],
        n if n.starts_with("user_") || n.starts_with("group_") || n.starts_with("acl_") => {
            &["admin"]
        }
        n if n.starts_with("account_lock") => &["admin"],
        n if n.starts_with("oauth_") => &["admin"],
        n if n.starts_with("saml_") => &["admin"],
        n if n.starts_with("ldap_") => &["admin"],
        n if n.starts_with("outbound_password_") => &["admin"],
        n if n.starts_with("jwt_") => &["admin"],
        n if n.starts_with("security_")
            || n.starts_with("keystore_")
            || n.starts_with("truststore_") =>
        {
            &["admin"]
        }
        n if n.starts_with("password_policy_") => &["admin"],
        n if n.starts_with("quiesce_") => &["admin"],

        // ── develop ─────────────────────────────────────────────
        "flow_service_create"
        | "put_node"
        | "service_invoke"
        | "document_type_create"
        | "mapset_value" => &["develop"],
        n if n.starts_with("flow_debug_") => &["develop"],
        n if n.starts_with("test_") || n.starts_with("mock_") => &["develop"],
        n if n.starts_with("doctype_gen_") || n.starts_with("sap_") => &["develop"],
        n if n.starts_with("ns_dep_") => &["develop"],
        n if n.starts_with("flatfile_") => &["develop"],
        "node_list" | "node_get" | "node_delete" | "folder_create" => &["develop"],
        n if n.starts_with("package_") => &["develop", "deploy"],

        // ── adapters ────────────────────────────────────────────
        n if n.starts_with("adapter_") => &["adapters"],
        n if n.starts_with("jdbc_") => &["adapters"],

        // ── messaging ───────────────────────────────────────────
        n if n.starts_with("jms_") || n.starts_with("jndi_") => &["messaging"],
        n if n.starts_with("mqtt_") => &["messaging"],
        n if n.starts_with("streaming_") => &["messaging"],
        n if n.starts_with("trigger_") => &["messaging"],
        n if n.starts_with("messaging_") => &["messaging"],

        // ── monitor ─────────────────────────────────────────────
        n if n.starts_with("server_") => &["monitor"],
        n if n.starts_with("audit_") => &["monitor"],
        n if n.starts_with("alert_") => &["monitor"],
        n if n.starts_with("health_") => &["monitor"],
        n if n.starts_with("logger_") => &["monitor"],
        n if n.starts_with("cache_") => &["monitor"],

        // ── deploy ──────────────────────────────────────────────
        n if n.starts_with("marketplace_") => &["deploy"],
        "install_jars" => &["deploy"],
        n if n.starts_with("remote_server_") => &["deploy"],
        n if n.starts_with("global_var_") => &["deploy"],

        // ── network ─────────────────────────────────────────────
        n if n.starts_with("port_") || n.starts_with("port_access_") => &["network"],
        n if n.starts_with("url_alias_") => &["network"],
        n if n.starts_with("sftp_") => &["network"],
        n if n.starts_with("proxy_") => &["network"],
        n if n.starts_with("ip_access_") => &["network"],
        n if n.starts_with("websocket_") => &["network"],
        n if n.starts_with("egw_") => &["network"],

        // ── web services ────────────────────────────────────────
        n if n.starts_with("ws_") || n.starts_with("rest_") || n.starts_with("openapi_") => {
            &["develop", "network"]
        }

        // ── default: everything else gets "develop"
        _ => &["develop"],
    }
}

/// Check if a tool should be included given the active scopes.
/// If `active_scopes` is empty, all tools are included.
pub fn is_tool_allowed(tool_name: &str, active_scopes: &[String]) -> bool {
    if active_scopes.is_empty() {
        return true; // no filtering
    }

    // "readonly" scope: only allow tools that are list/get/status (no mutations)
    if active_scopes.iter().any(|s| s == "readonly") && is_readonly_tool(tool_name) {
        return true;
    }

    let tool_scopes = tool_scope(tool_name);
    tool_scopes
        .iter()
        .any(|ts| active_scopes.iter().any(|as_| as_ == ts))
}

fn is_readonly_tool(name: &str) -> bool {
    name.ends_with("_list")
        || name.ends_with("_get")
        || name.ends_with("_report")
        || name.ends_with("_status")
        || name.ends_with("_info")
        || name.ends_with("_state")
        || name.ends_with("_stats")
        || name == "is_status"
        || name == "list_instances"
        || name == "node_list"
        || name == "node_get"
        || name == "service_invoke"
        || name.ends_with("_categories")
        || name.ends_with("_registries")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::WmServer;

    /// Scopes an operator may put in WM_SCOPES. Keep in sync with the README
    /// ("Available scopes") and the doc comment on `AppConfig::scopes`.
    const DOCUMENTED_SCOPES: &[&str] = &[
        "admin",
        "develop",
        "deploy",
        "adapters",
        "messaging",
        "monitor",
        "network",
        "readonly",
    ];

    /// Every tool the server actually registers, read from the live router so
    /// tools added later are covered without touching these tests.
    fn registered_tools() -> Vec<String> {
        WmServer::registered_tool_names()
    }

    #[test]
    fn router_exposes_tools() {
        // Guards the tests below against silently passing on an empty list.
        assert!(registered_tools().len() > 300);
    }

    #[test]
    fn every_package_tool_stays_in_the_deploy_scope() {
        // Package management is what the "deploy" scope exists for. A new match
        // arm listing individual package_* tools ahead of the generic one drops
        // them out of "deploy" silently -- nothing fails at startup.
        for tool in registered_tools() {
            if let Some(rest) = tool.strip_prefix("package_") {
                let scopes = tool_scope(&tool);
                assert!(
                    scopes.contains(&"deploy"),
                    "{tool} lost the \"deploy\" scope (got {scopes:?}); \
                     a package_* tool must stay reachable for deployment \
                     (offending suffix: {rest})"
                );
            }
        }
    }

    #[test]
    fn every_registered_tool_has_at_least_one_scope() {
        for tool in registered_tools() {
            assert!(
                !tool_scope(&tool).is_empty(),
                "{tool} maps to no scope, so WM_SCOPES can never expose it"
            );
        }
    }

    #[test]
    fn every_scope_in_use_is_documented() {
        for tool in registered_tools() {
            for scope in tool_scope(&tool) {
                assert!(
                    DOCUMENTED_SCOPES.contains(scope),
                    "{tool} uses undocumented scope {scope:?}; add it to the \
                     README scope list, the AppConfig::scopes doc comment and \
                     DOCUMENTED_SCOPES"
                );
            }
        }
    }

    #[test]
    fn admin_tools_never_leak_into_the_develop_scope() {
        for tool in registered_tools() {
            let scopes = tool_scope(&tool);
            if scopes.contains(&"admin") {
                assert!(
                    !scopes.contains(&"develop"),
                    "{tool} is admin-scoped but also reachable from \"develop\""
                );
            }
        }
    }

    #[test]
    fn unset_scopes_expose_every_tool() {
        for tool in registered_tools() {
            assert!(is_tool_allowed(&tool, &[]), "{tool} hidden with no filter");
        }
    }

    #[test]
    fn readonly_scope_hides_mutating_tools() {
        let readonly = vec!["readonly".to_string()];
        for tool in [
            "put_node",
            "node_delete",
            "package_delete",
            "user_add",
            "is_shutdown",
            "jdbc_pool_delete",
        ] {
            assert!(
                !is_tool_allowed(tool, &readonly),
                "{tool} mutates state but is exposed under the readonly scope"
            );
        }
        for tool in ["node_list", "node_get", "package_info", "server_stats"] {
            assert!(
                is_tool_allowed(tool, &readonly),
                "{tool} only reads state but is hidden under the readonly scope"
            );
        }
    }

    #[test]
    fn a_scope_only_matches_its_own_tools() {
        assert!(is_tool_allowed("put_node", &["develop".to_string()]));
        assert!(!is_tool_allowed("put_node", &["monitor".to_string()]));
        assert!(is_tool_allowed("user_add", &["admin".to_string()]));
        assert!(!is_tool_allowed("user_add", &["develop".to_string()]));
    }
}
