# MCP Server Gap Analysis: v2.5.0 (336 tools) vs Full Designer + IS Admin

## Current Coverage Summary

336 tools + 9 prompts + 5 RAG resources covering: flow services, adapters (JDBC/SAP/OPC), adapter metadata browsing, streaming/Kafka, JMS, MQTT, JNDI, scheduler, users/groups/ACLs, JDBC pools, global variables, server monitoring, remote servers, auditing, OAuth, web services/OpenAPI, security/keystores, ports, SFTP, proxy, JWT, quiesce, health indicators, alerts, enterprise gateway, IP access, password policy, WebSocket, marketplace, JAR installation, flow debugging, unit testing/mocking, namespace dependencies, flat file schemas, cache manager, SAML, LDAP, logger configuration, outbound passwords, port access control, messaging publish.

---

## TIER 1 -- HIGH PRIORITY (daily development/admin workflow)

### 1. Package Marketplace (packages.webmethods.io)
Browse, download, and install packages from the webMethods package registry.
- Browse available packages by category/keyword
- Download package zip
- Install package into IS via `pub.packages:installPackage` or `wm.server.packages:packageInstall`
- **Source**: https://packages.webmethods.io/

### 2. Pub/Sub Trigger Management
IS services: `wm.server.triggers:createTrigger`, `deleteTrigger`, `getProperties`, `setProperties`, `getTriggerReport`, `suspendTrigger`, `getTriggerStats`, `getProcessingStatus`, `getRetrievalStatus`

### 3. webMethods Messaging Connections (native, not JMS/MQTT)
REST API: `GET/POST/PATCH/DELETE /admin/messaging/`
IS services: `wm.server.messaging:createConnectionAlias`, `deleteConnectionAlias`, `enableConnectionAlias`, `disableConnectionAlias`, `getConnectionAliases`, `getConnectionAliasReport`
Plus: `wm.server.publish:publish`, `publishAndWait`, `deliver`

### 4. HTTP URL Aliases
REST API: `GET/POST/PATCH/PUT/DELETE /admin/urlalias/`
IS services: `wm.server.httpUrlAlias:addAlias`, `deleteAlias`, `getAlias`, `listAlias`, `updateAlias`

### 5. Flow Debugging
IS services: `wm.server.flowdebugger:start`, `invokeService`, `execute`, `insertBreakPoints`, `removeBreakPoints`, `getPipelineForBreakPoint`, `setPipeline`, `close`
Plus: `wm.server.flow:startFlow`, `stepFlow`, `executeFlowStep`, `pipe`, `endFlow`

### 6. Namespace Dependency Analysis
IS services: `wm.server.ns.dependency:getDependents`, `getReferences`, `getUnresolved`, `search`, `advancedSearch`, `refactor`, `refactorPreview`

### 7. Package Management Gaps
IS services: `wm.server.packages:packageDelete`, `packageInstall`, `packageInfo`, `packageSettings`, `compilePackage`, `addDepend`, `delDepend`, `getDependenciesList`, `packageAddStartupService`, `packageRemoveStartupService`, `jarUpload`, `jarList`, `jarDelete`

### 8. Document Type Generation (from XSD/JSON/XML/DTD)
IS services: `wm.server.record:generateFromXSDSource`, `generateFromJSONSchema`, `generateFromJSONString`, `generateFromXMLString`, `generateFromDTDString`

### 9. Unit Test Framework
IS services: `wm.ps.serviceMock:loadMock`, `clearMock`, `clearAllMocks`, `getMockedServices`, `suspendMocks`, `resumeMocks`
Plus: `wm.task.executor:run`, `runAdvanced`, `checkstatus`, `junitxmlreport`, `textreport`

### 10. Flat File Schema Creation
IS services: `pub.flatFile.generate:saveXMLAsFFSchema`, `createFFDictionary`, `getFFSchemaAsXML`, `deleteFFSchema`

**Estimated: ~95 tools**

---

## TIER 2 -- MEDIUM PRIORITY (admin operations, environment config)

### 11. SFTP Client Configuration
REST API: `GET/POST/PATCH/PUT/DELETE /admin/sftpserver/`, `/admin/sftpuser/`
IS services: `wm.server.sftpclient:createServerAlias`, `updateServerAlias`, `deleteServerAlias`, `listServerAliases`, `createUserAlias`, `updateUserAlias`, `removeUserAlias`, `listUserAliases`, `testConnection`

### 12. HTTP Proxy Configuration
REST API: `GET/POST/PATCH/PUT/DELETE /admin/proxy/`
IS services: `wm.server.proxy:createProxyServerAlias`, `deleteProxyServerAlias`, `enableProxyServerAlias`, `disableProxyServerAlias`, `getProxyServerAliases`

### 13. JWT Issuer Management
REST API: `GET/POST/PATCH/PUT/DELETE /admin/jwt/issuer/`, `GET/PATCH /admin/jwt/globalsettings`
IS services: `wm.server.jwt:addIssuer`, `removeIssuer`, `getIssuer`, `listIssuers`, `updateIssuer`, `getGlobalSettings`, `updateGlobalSettings`

### 14. Cache Manager
REST API: `GET/POST/PATCH/PUT/DELETE /admin/cachemanager/`, `/admin/cachemanager/{name}/cache`
IS services: `wm.server.cache:resetCache`

### 15. Port Access Control
IS services: `wm.server.portAccess:addNodes`, `deleteNode`, `getPort`, `portList`, `setType`, `resetPort`

### 16. Enterprise Gateway / Threat Protection
REST API: `GET/POST/PATCH/PUT/DELETE /admin/enterprisegateway/rule*`, `/admin/enterprisegateway/dos*`
IS services: `wm.server.enterprisegateway:addRule`, `deleteRule`, `updateRule`, `getRulesList`, `getDOS`, `saveDOS`

### 17. SAML Configuration
REST API: `GET/POST/PATCH/PUT/DELETE /admin/saml/`
IS services: `wm.server.saml:addIssuer`, `deleteIssuer`, `listIssuers`

### 18. LDAP Configuration
REST API: `GET/POST/PATCH/PUT/DELETE /admin/ldap/`, `/admin/ldap/dir/`
IS services: `wm.server.ldap:addConfiguredServer`, `editConfiguredServer`, `deleteConfiguredServer`, `getSettings`

### 19. Quiesce Mode
REST API: `GET/POST /admin/quiesce`
IS services: `wm.server.quiesce:setQuiesceMode`, `setActiveMode`, `getCurrentMode`

### 20. Health Indicators
REST API: `GET/PATCH/POST /admin/healthgauge/`
IS services: `wm.server.healthindicators:getAllHealthIndicators`, `getHealthIndicator`, `changeHealthIndicator`

### 21. Alert Management
REST API: `GET/POST /admin/alert`, `/admin/alert/channels`, `/admin/alert/notifier`
IS services: `wm.server.alert:enableNotifiers`, `disableAllNotifiers`, `alertingStatus`
Plus: `wm.server.event:addSubscriber`, `deleteSubscriber`, `getSubscribers`, `getEventTypes`

### 22. ACL Gaps (Assign to services, precedence)
IS services: `wm.server.access:aclAssign`, `getNodeNameListForAcl`, `getDefaultAccess`, `setDefaultAccess`

### 23. Password Policy & Account Locking Gaps
REST API: `GET/PATCH /admin/account/password/expiration`, `/admin/account/password/restriction`
IS services: `wm.server.access:updateAccountLockingSettings`, `resetAccountLockingSettings`, `listLockedAccounts`, `unlockAccount`, `getPasswordExpirySettings`, `updateExpirySettings`

### 24. Server Admin Gaps (IP access, thread management, sessions)
IS services: `wm.server.net:ipRuleAdd`, `ipRuleDelete`, `ipRuleList`, `changeIPAccessType`
Plus: `wm.server.query:interruptThread`, `killThread`, `wm.server.admin:killSession`, `clearSSLCache`

### 25. Outbound Passwords
IS services: `wm.server.outboundPasswords:storePassword`, `retrievePassword`, `removePassword`

### 26. WebSocket Server Management
IS services: `wm.server.net.websocket:createWebSocketEndpoint`, `listSessionsByPort`, `closeSession`, `broadcast`

### 27. Logger / Log Level Configuration
REST API: `GET/POST/PATCH/PUT /admin/logger/`, `/admin/logger/server*`

### 28. WS Endpoint CRUD Gaps
IS services: `wm.server.ws:addConsumerEndpoint`, `addProviderEndpoint`, `deleteConsumerEndpoint`, `deleteProviderEndpoint`, `refreshWSConnectors`

**Estimated: ~95 tools**

---

## TIER 3 -- LOW PRIORITY (specialized features)

### 29. CSRF Guard Configuration
### 30. VCS Integration (checkin/checkout)
### 31. Package Replication/Distribution
### 32. Deployer (ACDL-based deployment)
### 33. Thread Pool Configuration
### 34. MIME Types Configuration
### 35. Kerberos Configuration
### 36. OpenID Connect Settings
### 37. Data Collector / Metrics Engine
### 38. GraphQL Descriptor Management
### 39. Swagger (Legacy OpenAPI v2)
### 40. XA Recovery
### 41. OAuth Client Connections (outbound)
### 42. Hot Deployment Settings
### 43. XML Parsing/Security Settings
### 44. Cluster Management
### 45. gRPC Configuration
### 46. Reference Data Management
### 47. Asset Inventory & Sync
### 48. Client Certificate Management
### 49. Server SSO / Session / Thread Pool Config
### 50. Parquet File Processing
### 51. Guaranteed Delivery (pub.remote.gd)

**Estimated: ~70 tools**

---

## KEY INSIGHT: REST Admin API

The IS REST Admin API at `/admin/swagger/integrationServer` is an untapped gold mine. It provides a fully documented OpenAPI spec for ALL admin operations. Instead of reverse-engineering DSP pages and `wm.server.*` service signatures, we could:
1. Download the OpenAPI spec from a running IS
2. Auto-generate tool definitions from the spec
3. Get proper request/response schemas for free

This could potentially accelerate implementing Tier 2 and Tier 3 significantly.

---

## TOTAL GAP

| Tier | Status | Tools |
|---|---|---|
| **Current (v2.6.0)** | DONE | 336 tools |
| **Tier 1** | ~95% complete | Remaining: flat file advanced ops |
| **Tier 2** | ~90% complete | Remaining: event subscribers, alert channels |
| **Tier 3** | 23 areas | ~70 tools (CSRF, VCS, Replication, Deployer, etc.) |
| **Full parity** | | **~406+ tools** |
