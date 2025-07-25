import nodeResolve from "@rollup/plugin-node-resolve";
import { oveo } from "@oveo/rollup";

export default {
  input: "./app.js",
  output: {
    format: "es",
    strict: true,
    dir: "dist",
    generatedCode: "es2015",
  },
  watch: {
    chokidar: {
      usePolling: true,
    }
  },
  plugins: [
    nodeResolve(),
    oveo({
      hoist: true,
      dedupe: true,
      globals: {
        hoist: true,
        singletons: true,
      },
      externs: {
        inlineConstValues: true,
        import: [
          "./externs.json",
        ],
      },
      renameProperties: {
        pattern: "_$",
        map: "./properties.ini",
      },
    }),
  ],
};