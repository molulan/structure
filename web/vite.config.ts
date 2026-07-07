import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The SPA calls the API under `/api`, which the dev server proxies to the Axum
// backend (routes live at the root, so `/api` is stripped). The target is
// overridable so the e2e harness can point it at its own backend instance.
const proxyTarget = process.env.VITE_PROXY_TARGET ?? "http://127.0.0.1:3000";

export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    proxy: {
      "/api": {
        target: proxyTarget,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ""),
      },
    },
  },
});
