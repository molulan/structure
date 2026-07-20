import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";

// The SPA calls the API under `/api`, which the dev server proxies to the Axum
// backend — where the API is also mounted under `/api`, so the prefix is
// forwarded as-is. The target is overridable so the e2e harness can point it at
// its own backend instance.
const proxyTarget = process.env.VITE_PROXY_TARGET ?? "http://127.0.0.1:3000";

export default defineConfig({
  // The router plugin generates src/routeTree.gen.ts from src/routes/ and must
  // run before the React plugin.
  plugins: [tanstackRouter({ target: "react", autoCodeSplitting: true }), react()],
  server: {
    host: "127.0.0.1",
    proxy: {
      "/api": {
        target: proxyTarget,
        changeOrigin: true,
      },
    },
  },
});
