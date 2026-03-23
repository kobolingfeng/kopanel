import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  publicDir: "icons",  // 将icons目录作为公共资源目录
  base: "./",  // 使用相对路径，确保 Tauri 打包后能正确加载资源
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    chunkSizeWarningLimit: 550,
    rollupOptions: {
      input: {
        main: "index.html",
        osd: "osd.html",
      },
      output: {
        manualChunks(id) {
          const normalizedId = id.replaceAll("\\", "/");

          if (normalizedId.includes("/node_modules/")) {
            if (normalizedId.includes("/@tauri-apps/")) return "vendor-tauri";
            if (normalizedId.includes("/svelte/")) return "vendor-svelte";
            return "vendor";
          }

          if (normalizedId.includes("/src/lib/")) return "app-lib";
          if (normalizedId.includes("/src/components/")) return "app-components";
        },
      },
    },
  },
});
