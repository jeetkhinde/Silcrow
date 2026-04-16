#!/usr/bin/env node
"use strict";

const path = require("path");
const { buildDocsManifest, writeJsonFile } = require("../lib/build-docs");

const projectRoot = path.resolve(__dirname, "../..");
const outputPath = path.join(projectRoot, "mcp", "generated", "docs.json");
const manifest = buildDocsManifest({ projectRoot });

writeJsonFile(outputPath, manifest);

const sectionCount = manifest.docs.reduce((count, doc) => count + doc.sections.length, 0);
const htmlExampleCount = manifest.docs.reduce((count, doc) => count + doc.examples.html.length, 0);
const jsonExampleCount = manifest.docs.reduce((count, doc) => count + doc.examples.json.length, 0);

console.log(
  `Wrote ${path.relative(projectRoot, outputPath)}: ${manifest.docs.length} docs, ${sectionCount} sections, ${htmlExampleCount} HTML examples, ${jsonExampleCount} JSON examples`
);
