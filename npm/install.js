#!/usr/bin/env node
"use strict";

const { install } = require("./binary");

install().catch((err) => {
  console.error("Failed to install wm-mcp-server:", err.message);
  // Don't fail the npm install -- the binary can be downloaded on first run
  process.exit(0);
});
