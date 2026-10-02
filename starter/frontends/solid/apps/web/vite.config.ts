import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  // `npm run dev` serves the UI and forwards API calls to `cargo run`.
  server: { proxy: { "/api": "http://127.0.0.1:3000" } },
});
