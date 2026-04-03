#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");
const { platform, arch } = process;

const PKGS = {
  "darwin arm64": "@delimitter/mcp-darwin-arm64",
  "darwin x64":   "@delimitter/mcp-darwin-x64",
  "linux x64":    "@delimitter/mcp-linux-x64",
  "win32 x64":    "@delimitter/mcp-win32-x64",
};

const key = `${platform} ${arch}`;
const pkg = PKGS[key];

if (!pkg) {
  process.stderr.write(
    `synoema-mcp: unsupported platform ${platform}/${arch}\n` +
    `Build from source: https://github.com/Delimitter/synoema\n`
  );
  process.exit(1);
}

const binName = platform === "win32" ? "synoema-mcp.exe" : "synoema-mcp";

let bin;
try {
  // Resolve package directory via package.json (works for non-JS files)
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  bin = path.join(pkgDir, binName);
} catch {
  process.stderr.write(
    `synoema-mcp: platform package ${pkg} is not installed\n` +
    `Try reinstalling: npm install synoema-mcp\n`
  );
  process.exit(1);
}

if (!fs.existsSync(bin)) {
  process.stderr.write(
    `synoema-mcp: binary not found at ${bin}\n` +
    `The package may be installed without a binary — try a tagged release.\n`
  );
  process.exit(1);
}

const { status, error } = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (error) {
  process.stderr.write(`synoema-mcp: failed to start: ${error.message}\n`);
  process.exit(1);
}
process.exit(status ?? 0);
