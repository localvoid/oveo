import { defineConfig } from "vite";
import { oveo } from "@oveo/vite";

export default defineConfig({
  build: {
    minify: false,
  },
  plugins: [
    oveo({
      hoist: true,
      dedupe: true,
      globals: {
        include: ["js", "web"],
        hoist: true,
        singletons: true,
      },
      externs: {
        import: ["externs.json"],
        inlineConstValues: true,
      },
      renameProperties: {
        map: "properties.ini",
      },
    }),
  ],
});
