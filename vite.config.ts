import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { codeInspectorPlugin } from "code-inspector-plugin";

export default defineConfig({
  plugins: [codeInspectorPlugin({ bundler: "vite" }), solid()],
  server: {
    port: 3420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  clearScreen: false,
});
