#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { validateDocsManifest } = require("../lib/validate-docs");

const projectRoot = path.resolve(__dirname, "../..");
const docsJsonPath = path.join(projectRoot, "mcp", "generated", "docs.json");

if (!fs.existsSync(docsJsonPath)) {
  console.error("mcp/generated/docs.json is missing. Run npm run build:docs.");
  process.exit(1);
}

const manifest = JSON.parse(fs.readFileSync(docsJsonPath, "utf8"));
const errors = validateDocsManifest(manifest);

if (errors.length > 0) {
  console.error("docs.json validation failed:");
  for (const error of errors) console.error(`- ${error}`);
  process.exit(1);
}

console.log(`docs.json is valid: ${manifest.docs.length} docs`);
