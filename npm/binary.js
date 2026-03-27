#!/usr/bin/env node
"use strict";

const os = require("os");
const path = require("path");
const fs = require("fs");

const VERSION = require("./package.json").version;
const REPO = "cpoder/wm-mcp-server";

/**
 * Map Node.js platform/arch to GitHub Release asset names.
 */
function getAssetName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    "linux-x64": "wm-mcp-server-linux-x86_64",
    "linux-arm64": "wm-mcp-server-linux-aarch64",
    "darwin-x64": "wm-mcp-server-macos-x86_64",
    "darwin-arm64": "wm-mcp-server-macos-aarch64",
    "win32-x64": "wm-mcp-server-windows-x86_64.exe",
  };

  const key = `${platform}-${arch}`;
  const asset = map[key];
  if (!asset) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Supported: ${Object.keys(map).join(", ")}`
    );
  }
  return asset;
}

/**
 * Returns the URL to download the binary from GitHub Releases.
 */
function getDownloadUrl() {
  const asset = getAssetName();
  return `https://github.com/${REPO}/releases/download/v${VERSION}/${asset}`;
}

/**
 * Returns the local path where the binary is stored.
 */
function getBinaryPath() {
  const name = os.platform() === "win32" ? "wm-mcp-server.exe" : "wm-mcp-server";
  return path.join(__dirname, name);
}

/**
 * Download a URL, following redirects (GitHub releases redirect to S3).
 * Returns a Promise<Buffer>.
 */
function download(url, maxRedirects = 5) {
  return new Promise((resolve, reject) => {
    if (maxRedirects <= 0) {
      return reject(new Error("Too many redirects"));
    }

    const lib = url.startsWith("https") ? require("https") : require("http");
    lib.get(url, { headers: { "User-Agent": "wm-mcp-server-npm" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return resolve(download(res.headers.location, maxRedirects - 1));
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`Download failed: HTTP ${res.statusCode} from ${url}`));
      }

      const chunks = [];
      res.on("data", (chunk) => chunks.push(chunk));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    }).on("error", reject);
  });
}

/**
 * Install the binary: download from GitHub Releases and save to disk.
 */
async function install() {
  const binPath = getBinaryPath();

  if (fs.existsSync(binPath)) {
    console.log(`wm-mcp-server binary already exists at ${binPath}`);
    return;
  }

  const url = getDownloadUrl();
  console.log(`Downloading wm-mcp-server v${VERSION}...`);
  console.log(`  ${url}`);

  const data = await download(url);
  fs.writeFileSync(binPath, data);

  if (os.platform() !== "win32") {
    fs.chmodSync(binPath, 0o755);
  }

  console.log(`Installed wm-mcp-server to ${binPath}`);
}

/**
 * Run the binary, forwarding all arguments and stdio.
 */
function run() {
  const binPath = getBinaryPath();

  if (!fs.existsSync(binPath)) {
    console.error(
      "wm-mcp-server binary not found. Run 'npm install' or 'node install.js' first."
    );
    process.exit(1);
  }

  const { spawnSync } = require("child_process");
  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });

  process.exit(result.status ?? 1);
}

module.exports = { install, run, getBinaryPath, getDownloadUrl };
