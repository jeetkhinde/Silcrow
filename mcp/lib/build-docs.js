"use strict";

const fs = require("fs");
const path = require("path");
const { transformMarkdownDoc } = require("./transform-doc");

function buildDocsManifest(options = {}) {
  const projectRoot = options.projectRoot || path.resolve(__dirname, "../..");
  const packageJson = readPackageJson(projectRoot);
  const docsDir = options.docsDir || path.join(projectRoot, "docs");
  const files = listMarkdownFiles(docsDir);

  return {
    schema_version: 1,
    project: {
      id: packageJson.name,
      name: "Silcrow.js",
      version: packageJson.version,
    },
    generated_at: new Date().toISOString(),
    docs: files.map((fileName) =>
      transformMarkdownDoc({
        fileName,
        markdown: fs.readFileSync(path.join(docsDir, fileName), "utf8"),
      })
    ),
  };
}

function listMarkdownFiles(docsDir) {
  return fs
    .readdirSync(docsDir, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .filter((fileName) => fileName.toLowerCase().endsWith(".md"))
    .sort((a, b) => a.localeCompare(b));
}

function readPackageJson(projectRoot) {
  return JSON.parse(fs.readFileSync(path.join(projectRoot, "package.json"), "utf8"));
}

function writeJsonFile(filePath, data) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`);
}

module.exports = {
  buildDocsManifest,
  listMarkdownFiles,
  writeJsonFile,
};
