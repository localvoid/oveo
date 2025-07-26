import nodeResolve from "@rollup/plugin-node-resolve";
import { oveo } from "@oveo/rollup";

export default {
  input: "./src/main.js",
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
        include: ["js", "web"],
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
        pattern: "^[^_].+[^_]_$",
        map: "./properties.ini",
      },
    }),
  ],
};