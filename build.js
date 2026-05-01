
const fs = require("fs");
const path = require("path");
const {minify} = require("terser");
const zlib = require("zlib");


const srcDir = path.join(__dirname, "src");
const srcFile = path.join(srcDir, "silcrow.js");
const distDir = path.join(__dirname, "dist");
const distFile = path.join(distDir, "silcrow.js");
const minFile = path.join(distDir, "silcrow.min.js");
const pilcrowRuntimeAssetsDir = path.resolve(
  __dirname,
  "../pilcrow/crates/runtime/assets"
);

async function build() {
  const source = fs.readFileSync(srcFile, "utf8");

  const banner = `// Silcrow.js — Hypermedia Runtime\n// Built: ${new Date().toISOString()}`;

  const minResult = await minify(source, {
    ecma: 2020,
    compress: {
      passes: 2,
      unsafe: false,
      drop_console: true,
    },
    mangle: {
      toplevel: true,
    },
    format: {
      preamble: banner,
      comments: false,
    },
  });

  fs.copyFileSync(srcFile, distFile);

  const minified = minResult.code;
  fs.writeFileSync(minFile, minified);

  const rawKb = (Buffer.byteLength(minified) / 1024).toFixed(1);
  const gzKb = (zlib.gzipSync(minified).length / 1024).toFixed(1);
  const brKb = (zlib.brotliCompressSync(minified).length / 1024).toFixed(1);

  // Log results
  // copied file from src to dist as it is, then log the minified file size and the gzipped/brotli sizes
  console.log(`✓ ${path.relative(__dirname, distFile)} ${rawKb} KB  (copied from ${path.relative(__dirname, srcFile)})`);

  console.log(`✓ ${path.relative(__dirname, minFile)} (${rawKb} KB raw | ${gzKb} KB gzip | ${brKb} KB brotli)`);

  // Mirror minified runtime into Pilcrow assets
  if (fs.existsSync(pilcrowRuntimeAssetsDir)) {
    const pilcrowAssetFile = path.join(pilcrowRuntimeAssetsDir, "silcrow.js");
    fs.copyFileSync(minFile, pilcrowAssetFile);
    console.log(`✓ ${path.relative(__dirname, pilcrowAssetFile)} (from dist/silcrow.min.js)`);
  }
}

build().then(() => {
  if (process.argv.includes("--watch")) {
    console.log("\nWatching dist/silcrow.js for changes...");
    fs.watch(srcFile, () => {
      console.log("\n⟳ silcrow.js changed, rebuilding...");
      build().catch(console.error);
    });
  }
}).catch(err => {
  console.error("Build failed:", err);
  process.exit(1);
});
