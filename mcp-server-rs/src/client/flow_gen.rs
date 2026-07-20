use serde_json::json;

impl super::ISClient {
    // ── DSL Validation ──────────────────────────────────────────────────
    // wm.dsl:validate checks DSL source text against its grammar without
    // deploying anything. FSL is currently the only DSL implementation on
    // IS, but the service is DSL-agnostic — more DSLs may be added later.

    pub async fn dsl_validate(&self, dsl_text: &str) -> Result<serde_json::Value, String> {
        self.invoke_post("wm.dsl:validate", &json!({"dslText": dsl_text}))
            .await
    }

    // ── FSL Deployment ───────────────────────────────────────────────────
    // wm.server.flowGen:generateFromFSLString compiles FSL source and
    // deploys the resulting flow service on IS in a single atomic call.
    // The returned `node` field is IS's internal Values/FlowElement tree —
    // it is informational only. NEVER repost it via put_node: round-tripping
    // it through IS's generic JSON REST serialization flattens the nested
    // `flow.nodes` array into a string (e.g. "[INVOKE]"), silently producing
    // a service with no logic. generateFromFSLString already deploys the
    // service; no follow-up put_node call is needed or safe.
    //
    // packageName must already exist on IS (use package_create first) —
    // this call does not create packages implicitly.

    pub async fn fsl_deploy(
        &self,
        fsl_string: &str,
        package_name: &str,
        ifc_name: &str,
        flow_name: &str,
    ) -> Result<serde_json::Value, String> {
        self.invoke_post(
            "wm.server.flowGen:generateFromFSLString",
            &json!({
                "fslString": fsl_string,
                "packageName": package_name,
                "ifcName": ifc_name,
                "flowName": flow_name,
            }),
        )
        .await
    }

    // ── FSL Extraction ───────────────────────────────────────────────────
    // wm.server.flowGen:generateFSLFromFlow decompiles an existing flow
    // service back into FSL source. The regenerated FSL is semantically
    // equivalent but not always textually identical to any FSL originally
    // used to create the service (e.g. it may add an explicit `properties`
    // block or reformat mappings). On failure for a nonexistent service,
    // the IS-side `message` may be a raw internal exception string rather
    // than a clean validation message — surface it as-is.

    pub async fn fsl_extract(&self, service_name: &str) -> Result<serde_json::Value, String> {
        self.invoke_post(
            "wm.server.flowGen:generateFSLFromFlow",
            &json!({"serviceName": service_name}),
        )
        .await
    }
}
