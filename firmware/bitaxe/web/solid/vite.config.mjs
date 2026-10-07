// Vite build for the SolidJS operator UI variant. Bazel runs it through
// //firmware/bitaxe/web/solid:dist; the staging step then adds gzip siblings.
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  base: "/",
  plugins: [solid()],
  // Bazel sandboxes link inputs; resolving through the links would move
  // index.html outside the project root.
  resolve: { preserveSymlinks: true },
  build: {
    outDir: "dist",
    assetsDir: "assets",
    emptyOutDir: true,
    // Hashed names keep a 30-day cached older bundle from running after an
    // OTAWWW swap; index.html stays unhashed and is served without caching.
    assetsInlineLimit: 0,
    modulePreload: { polyfill: false },
    reportCompressedSize: false,
    sourcemap: false,
    target: "es2022",
  },
});
