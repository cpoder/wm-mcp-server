"""
webMethods Integration Server HTTP Client

Pure HTTP client for interacting with the IS REST API.
All operations use the IS HTTP API - no disk access required.
"""

import httpx
import json
from typing import Any, Optional


class ISClient:
    """HTTP client for webMethods Integration Server."""

    def __init__(
        self,
        base_url: str = "http://localhost:5555",
        username: str = "Administrator",
        password: str = "manage",
        timeout: float = 30.0,
    ):
        self.base_url = base_url.rstrip("/")
        self.auth = httpx.BasicAuth(username, password)
        self.timeout = timeout

    def _client(self) -> httpx.AsyncClient:
        return httpx.AsyncClient(
            base_url=self.base_url,
            auth=self.auth,
            timeout=self.timeout,
            verify=False,
            headers={"Accept": "application/json"},
        )

    # ── Server Management ──────────────────────────────────────────────

    async def is_running(self) -> bool:
        """Check if Integration Server is running."""
        try:
            async with self._client() as client:
                r = await client.get("/invoke/wm.server.packages/packageList")
                return r.status_code == 200
        except Exception:
            return False

    async def get_server_status(self) -> dict:
        """Get server status information."""
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.admin/getServerStatus")
            r.raise_for_status()
            return r.json()

    async def shutdown(self, bounce: bool = False) -> dict:
        """Shutdown the Integration Server via HTTP API.

        Args:
            bounce: If True, the server will restart (bounce) instead of stopping.
        """
        async with self._client() as client:
            params = {}
            if bounce:
                params["bounce"] = "yes"
            r = await client.get("/invoke/wm.server.admin/shutdown", params=params)
            r.raise_for_status()
            return {"status": "shutdown initiated", "bounce": bounce}

    # ── Package Management ─────────────────────────────────────────────

    async def package_list(self) -> dict:
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.packages/packageList")
            r.raise_for_status()
            return r.json()

    async def package_create(self, package_name: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.packages/packageCreate",
                json={"package": package_name},
            )
            r.raise_for_status()
            data = r.json() if r.text.strip() else {}
            # Auto-activate
            try:
                await client.get(
                    "/invoke/wm.server.packages/packageActivate",
                    params={"package": package_name},
                )
            except Exception:
                pass
            return {"status": "created", "package": package_name, "message": data.get("message", "")}

    async def package_reload(self, package_name: str) -> dict:
        async with self._client() as client:
            r = await client.get(
                "/invoke/wm.server.packages/packageReload",
                params={"package": package_name},
            )
            r.raise_for_status()
            return {"status": "reloaded", "package": package_name}

    async def package_enable(self, package_name: str) -> dict:
        async with self._client() as client:
            r = await client.get(
                "/invoke/wm.server.packages/packageEnable",
                params={"package": package_name},
            )
            r.raise_for_status()
            return {"status": "enabled", "package": package_name}

    async def package_disable(self, package_name: str) -> dict:
        async with self._client() as client:
            r = await client.get(
                "/invoke/wm.server.packages/packageDisable",
                params={"package": package_name},
            )
            r.raise_for_status()
            return {"status": "disabled", "package": package_name}

    # ── Namespace / Node Management ────────────────────────────────────

    async def node_list(self, package: str, interface: str = "") -> dict:
        params = {"package": package}
        if interface:
            params["interface"] = interface
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.ns/getNodeList", params=params)
            r.raise_for_status()
            return r.json()

    async def node_get(self, name: str) -> dict:
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.ns/getNode", params={"name": name})
            r.raise_for_status()
            return r.json()

    async def node_delete(self, name: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.ns/deleteNode",
                json={"node_nsName": name},
            )
            r.raise_for_status()
            return {"status": "deleted", "node": name}

    async def folder_create(self, package: str, folder_path: str) -> dict:
        """Create a folder (interface) in a package."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.ns/makeNode",
                json={
                    "node_type": "interface",
                    "node_nsName": folder_path,
                    "node_pkg": package,
                },
            )
            r.raise_for_status()
            return {"status": "created", "folder": folder_path, "package": package}

    # ── Service Management via putNode ─────────────────────────────────

    async def put_node(self, node_data: dict) -> dict:
        """Create or update any namespace node via the putNode API.

        This is the core API for creating/updating flow services, document types, etc.
        The node_data dict follows the IS Values serialization format.
        """
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.ns/putNode",
                json=node_data,
            )
            r.raise_for_status()
            return {"status": "ok", "response": r.text[:500]}

    async def service_create(self, package: str, service_path: str) -> dict:
        """Create an empty flow service via serviceAdd API."""
        if ":" in service_path:
            interface_part, service_name = service_path.rsplit(":", 1)
        else:
            interface_part = ""
            service_name = service_path

        payload = {
            "service": service_name,
            "package": package,
            "serviceType": "flow",
        }
        if interface_part:
            payload["interface"] = interface_part

        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.services/serviceAdd",
                json=payload,
            )
            r.raise_for_status()
            data = r.json() if r.text.strip() else {}
            return {"status": "created", "service": service_path, "package": package, "message": data.get("message", "")}

    async def service_invoke(self, service_path: str, inputs: Optional[dict] = None) -> dict:
        """Invoke/execute a service."""
        async with self._client() as client:
            if inputs:
                r = await client.post(f"/invoke/{service_path}", json=inputs)
            else:
                r = await client.get(f"/invoke/{service_path}")
            r.raise_for_status()
            return r.json() if r.text.strip() else {"status": "invoked"}

    # ── Document Type Management ───────────────────────────────────────

    async def document_type_create(self, package: str, doc_path: str) -> dict:
        """Create a document type via makeNode."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.ns/makeNode",
                json={
                    "node_type": "record",
                    "node_nsName": doc_path,
                    "node_pkg": package,
                },
            )
            r.raise_for_status()
            return {"status": "created", "document": doc_path, "package": package}

    # ── Port / Listener Management ───────────────────────────────────

    async def port_list(self) -> dict:
        """List all ports/listeners (HTTP, HTTPS, FTP, FilePolling, etc.)."""
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.net.listeners/listListeners")
            r.raise_for_status()
            return r.json()

    async def port_factory_list(self) -> dict:
        """List available listener factories (HTTP, FTP, FilePolling, etc.)."""
        async with self._client() as client:
            r = await client.get("/invoke/wm.server.net.listeners/listFactories")
            r.raise_for_status()
            return r.json()

    async def port_get(self, port_key: str, pkg: str) -> dict:
        """Get details of a specific port/listener."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/getListener",
                json={"listenerKey": port_key, "pkg": pkg},
            )
            r.raise_for_status()
            return r.json()

    async def port_add(self, settings: dict) -> dict:
        """Add a new port/listener."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/addListener",
                json=settings,
            )
            r.raise_for_status()
            data = r.json() if r.text.strip() else {}
            return {"status": "created", "message": data.get("message", ""), "listenerKey": data.get("listenerKey", "")}

    async def port_update(self, listener_key: str, pkg: str, settings: dict) -> dict:
        """Update an existing port/listener."""
        payload = {"listenerKey": listener_key, "pkg": pkg}
        payload.update(settings)
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/updateListener",
                json=payload,
            )
            r.raise_for_status()
            return {"status": "updated", "listener": listener_key}

    async def port_enable(self, port_key: str, pkg: str) -> dict:
        """Enable a port/listener."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/enableListener",
                json={"listenerKey": port_key, "pkg": pkg},
            )
            r.raise_for_status()
            return {"status": "enabled", "listener": port_key}

    async def port_disable(self, port_key: str, pkg: str) -> dict:
        """Disable a port/listener."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/disableListener",
                json={"listenerKey": port_key, "pkg": pkg},
            )
            r.raise_for_status()
            return {"status": "disabled", "listener": port_key}

    async def port_delete(self, port_key: str, pkg: str) -> dict:
        """Delete a port/listener."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.server.net.listeners/deleteListener",
                json={"listenerKey": port_key, "pkg": pkg},
            )
            r.raise_for_status()
            return {"status": "deleted", "listener": port_key}

    # ── Adapter Connection Management ──────────────────────────────────

    async def adapter_connection_list(self) -> dict:
        try:
            async with self._client() as client:
                r = await client.get("/invoke/wm.art.admin.connection:listAllResources")
                r.raise_for_status()
                return r.json()
        except Exception as e:
            return {"error": str(e)}

    async def adapter_connection_create(
        self, connection_alias: str, package_name: str,
        adapter_type: str, connection_factory_type: str,
        connection_settings: dict, connection_manager_settings: dict = None,
    ) -> dict:
        if connection_manager_settings is None:
            connection_manager_settings = {
                "poolable": "true", "minimumPoolSize": "1", "maximumPoolSize": "10",
                "poolIncrementSize": "1", "blockingTimeout": "1000", "expireTimeout": "1000",
                "startupRetryCount": "0", "startupBackoffSecs": "5",
            }
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.art.dev.connection:createConnectionNode",
                json={
                    "connectionAlias": connection_alias,
                    "packageName": package_name,
                    "adapterTypeName": adapter_type,
                    "connectionFactoryType": connection_factory_type,
                    "connectionSettings": connection_settings,
                    "connectionManagerSettings": connection_manager_settings,
                },
            )
            r.raise_for_status()
            return {"status": "created", "connection": connection_alias, "response": r.text[:500]}

    async def adapter_connection_enable(self, connection_alias: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/pub.art.connection:enableConnection",
                json={"connectionAlias": connection_alias},
            )
            r.raise_for_status()
            return {"status": "enabled", "connection": connection_alias}

    async def adapter_connection_disable(self, connection_alias: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/pub.art.connection:disableConnection",
                json={"connectionAlias": connection_alias},
            )
            r.raise_for_status()
            return {"status": "disabled", "connection": connection_alias}

    async def adapter_connection_state(self, connection_alias: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/pub.art.connection:queryConnectionState",
                json={"connectionAlias": connection_alias},
            )
            r.raise_for_status()
            return r.json()

    async def adapter_type_list(self) -> dict:
        """List all registered adapter types."""
        async with self._client() as client:
            r = await client.get("/invoke/wm.art.admin:retrieveAdapterTypesList")
            r.raise_for_status()
            return r.json()

    async def adapter_connection_metadata(self, adapter_type: str, factory_type: str) -> dict:
        """Get metadata for creating connections of a specific adapter type."""
        async with self._client() as client:
            r = await client.post(
                "/invoke/wm.art.dev.connection:fetchConnectionMetadata",
                json={"adapterTypeName": adapter_type, "connectionFactoryType": factory_type},
            )
            r.raise_for_status()
            return r.json()

    async def adapter_listener_list(self, adapter_type: str) -> dict:
        try:
            async with self._client() as client:
                r = await client.post(
                    "/invoke/pub.art.listener:listAdapterListeners",
                    json={"adapterTypeName": adapter_type},
                )
                r.raise_for_status()
                return r.json()
        except Exception as e:
            return {"error": str(e)}

    async def adapter_listener_create(
        self, listener_alias: str, package_name: str,
        adapter_type: str, connection_alias: str,
        listener_settings: dict = None,
    ) -> dict:
        payload = {
            "listenerAlias": listener_alias, "packageName": package_name,
            "adapterTypeName": adapter_type, "connectionAlias": connection_alias,
        }
        if listener_settings:
            payload["listenerSettings"] = listener_settings
        async with self._client() as client:
            r = await client.post("/invoke/wm.art.dev.listener:createListenerNode", json=payload)
            r.raise_for_status()
            return {"status": "created", "listener": listener_alias, "response": r.text[:500]}

    async def adapter_listener_enable(self, listener_alias: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/pub.art.listener:enableListener",
                json={"listenerAlias": listener_alias},
            )
            r.raise_for_status()
            return {"status": "enabled", "listener": listener_alias}

    async def adapter_listener_disable(self, listener_alias: str) -> dict:
        async with self._client() as client:
            r = await client.post(
                "/invoke/pub.art.listener:disableListener",
                json={"listenerAlias": listener_alias},
            )
            r.raise_for_status()
            return {"status": "disabled", "listener": listener_alias}

    async def adapter_notification_list(self, adapter_type: str) -> dict:
        try:
            async with self._client() as client:
                r = await client.post(
                    "/invoke/pub.art.notification:listAdapterPollingNotifications",
                    json={"adapterTypeName": adapter_type},
                )
                r.raise_for_status()
                return r.json()
        except Exception as e:
            return {"error": str(e)}

    async def adapter_service_create(
        self, service_name: str, package_name: str,
        connection_alias: str, service_template: str,
        adapter_service_settings: dict = None,
    ) -> dict:
        """Create an adapter service via WmART dev API."""
        payload = {
            "serviceName": service_name, "packageName": package_name,
            "connectionAlias": connection_alias, "serviceTemplate": service_template,
        }
        if adapter_service_settings:
            payload["adapterServiceSettings"] = adapter_service_settings
        async with self._client() as client:
            r = await client.post("/invoke/wm.art.dev.service:createAdapterServiceNode", json=payload)
            r.raise_for_status()
            return {"status": "created", "service": service_name, "response": r.text[:500]}

    async def adapter_notification_create_polling(
        self, notification_name: str, package_name: str,
        connection_alias: str, notification_template: str,
        notification_settings: dict = None,
    ) -> dict:
        payload = {
            "notificationName": notification_name, "packageName": package_name,
            "connectionAlias": connection_alias, "notificationTemplate": notification_template,
        }
        if notification_settings:
            payload["notificationSettings"] = notification_settings
        async with self._client() as client:
            r = await client.post("/invoke/wm.art.dev.notification:createPollingNotificationNode", json=payload)
            r.raise_for_status()
            return {"status": "created", "notification": notification_name, "response": r.text[:500]}

    async def adapter_notification_create_listener(
        self, notification_name: str, package_name: str,
        listener_alias: str, notification_template: str,
        notification_settings: dict = None,
    ) -> dict:
        payload = {
            "notificationName": notification_name, "packageName": package_name,
            "listenerAlias": listener_alias, "notificationTemplate": notification_template,
        }
        if notification_settings:
            payload["notificationSettings"] = notification_settings
        async with self._client() as client:
            r = await client.post("/invoke/wm.art.dev.notification:createListenerNotificationNode", json=payload)
            r.raise_for_status()
            return {"status": "created", "notification": notification_name, "response": r.text[:500]}
