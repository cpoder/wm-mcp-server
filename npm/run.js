#!/usr/bin/env node
"use strict";

const fs = require("fs");
const { run, install, getBinaryPath } = require("./binary");

// If binary is missing (e.g., postinstall was skipped), download on first run
if (!fs.existsSync(getBinaryPath())) {
  install()
    .then(() => run())
    .catch((err) => {
      console.error("Failed to download wm-mcp-server:", err.message);
      process.exit(1);
    });
} else {
  run();
}
