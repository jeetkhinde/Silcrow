
const fs = require("fs");
const path = require("path");

// ── Source files in dependency order ────────────────────────
const SOURCE_ORDER = [
  "debug.js",
  "url-safety.js",
  "safety.js",
  "toasts.js",
  "atoms.js",
  "patcher.js",
  "live.js",
  "ws.js",
  "navigator.js",
  "optimistic.js",
  "index.js",
];

const srcDir = path.join(__dirname, "src");
const distDir = path.join(__dirname, "dist");
const pilcrowRuntimeAssetsDir = path.resolve(
  __dirname,
  "../pilcrow/crates/runtime/assets"
);

// ── Concatenate sources ────────────────────────────────────
const parts = SOURCE_ORDER.map((file) => {
  const filePath = path.join(srcDir, file);
  if (!fs.existsSync(filePath)) {
    console.error(`Missing source file: ${file}`);
    process.exit(1);
  }
  return fs.readFileSync(filePath, "utf8");
});

const banner = `// Silcrow.js — Hypermedia Runtime\n// Built: ${new Date().toISOString()}\n`;
const concatenated = parts.join("\n");
const wrapped = `${banner}(function(){\n"use strict";\n${concatenated}\n})();\n`;

// ── Ensure dist/ exists ────────────────────────────────────
fs.mkdirSync(distDir, { recursive: true });

// ── Write unminified bundle ────────────────────────────────
const outFile = path.join(distDir, "silcrow.js");
fs.writeFileSync(outFile, wrapped);
console.log(`✓ dist/silcrow.js (${(Buffer.byteLength(wrapped) / 1024).toFixed(1)} KB)`);

// ── Write minified bundle via Terser ──────────────────────
const { minify } = require("terser");
const zlib = require("zlib");

(async function finalize() {
  const minResult = await minify(wrapped, {
    ecma: 2020,
    compress: {
      passes:2,
      unsafe: false,
      drop_console: true
    },
    mangle: {
      toplevel: true,
    },
    format: {
      preamble: banner.trim(),
      comments: false
    }
  });

  const minified = minResult.code;
  const minFile = path.join(distDir, "silcrow.min.js");
  fs.writeFileSync(minFile, minified);

  const rawKb = (Buffer.byteLength(minified) / 1024).toFixed(1);
  const gzKb = (zlib.gzipSync(minified).length / 1024).toFixed(1);
  const brKb = (zlib.brotliCompressSync(minified).length / 1024).toFixed(1);

  console.log(`✓ dist/silcrow.min.js (${rawKb} KB raw | ${gzKb} KB gzip | ${brKb} KB brotli)`);

  // ── Mirror minified runtime into Pilcrow assets ────────────
  fs.mkdirSync(pilcrowRuntimeAssetsDir, { recursive: true });
  const pilcrowAssetFile = path.join(pilcrowRuntimeAssetsDir, "silcrow.js");
  fs.copyFileSync(minFile, pilcrowAssetFile);
  console.log(`✓ ${path.relative(__dirname, pilcrowAssetFile)} (from dist/silcrow.min.js)`);

  // ── Watch mode ─────────────────────────────────────────────
  if (process.argv.includes("--watch")) {
    console.log("\nWatching src/ for changes...");
    fs.watch(srcDir, { recursive: true }, (event, filename) => {
      if (!filename?.endsWith(".js")) return;
      console.log(`\n⟳ ${filename} changed, rebuilding...`);
      try {
        require("child_process").execSync("node build.js", {
          stdio: "inherit",
          cwd: __dirname,
        });
      } catch (e) {
        console.error("Build failed:", e.message);
      }
    });
  }
})().catch(err => { console.error("Minification failed:", err); process.exit(1); });
