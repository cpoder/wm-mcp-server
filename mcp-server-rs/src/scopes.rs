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
        "account_locking_get" => &["admin"],
        n if n.starts_with("oauth_") => &["admin"],
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

        // ── deploy ──────────────────────────────────────────────
        n if n.starts_with("marketplace_") => &["deploy"],
        "install_jars" => &["deploy"],
        n if n.starts_with("remote_server_") => &["deploy"],
        n if n.starts_with("global_var_") => &["deploy"],

        // ── network ─────────────────────────────────────────────
        n if n.starts_with("port_") => &["network"],
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
