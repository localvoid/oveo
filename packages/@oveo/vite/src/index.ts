import * as fs from "node:fs/promises";
import type { Plugin, ResolvedId } from "rollup";
import { createFilter, type FilterPattern } from "@rollup/pluginutils";
import { Optimizer, type OptimizerOptions } from "@oveo/optimizer";

export interface PluginOptions extends OptimizerOptions {
  readonly include?: FilterPattern | undefined;
  readonly exclude?: FilterPattern | undefined;
  readonly externs?: { inlineConstValues?: boolean, import?: string[]; },
  readonly renameProperties?: { pattern?: string, map?: string; },
}

// `Plugin & { apply?: "build"; }` is a workaround to avoid installing Vite with
// a lot of dependencies.
export function oveo(options: PluginOptions = {}): Plugin & { apply?: "build"; } {
  const filter = createFilter(
    options?.include ?? /\.(m?js|m?tsx?)$/,
    options?.exclude,
  );
  const externs = new Set<string>();

  let reloadExterns = new Set<string>();
  let init = false;
  let propertyMap: ResolvedId | null = null;

  if (options.externs?.import !== void 0) {
    for (const extern of options.externs.import) {
      externs.add(extern);
      reloadExterns.add(extern);
    }
  }

  const opt = new Optimizer(options);
  return {
    name: "oveo:optimizer",
    apply: "build",

    async buildStart() {
      if (!init) {
        init = true;
        if (options.renameProperties?.map !== void 0) {
          propertyMap = await this.resolve(options.renameProperties.map);
          if (propertyMap) {
            try {
              opt.importPropertyMap(await fs.readFile(propertyMap.id));
            } catch (err) {
              this.error(err);
            }
          } else {
            this.warn(`Unable to find property map '${options.renameProperties.map}'"`);
          }
        }

      }
      if (reloadExterns.size > 0) {
        for (const extern of reloadExterns) {
          try {
            const resolved = await this.resolve(extern);
            if (resolved) {
              this.addWatchFile(resolved.id);
              const data = await fs.readFile(resolved.id);
              opt.importExterns(data);
            } else {
              this.warn(`Unable to find extern file '${extern}'`);
            }
          } catch (err) {
            this.error(err);
          }
        }
        reloadExterns.clear();
      }
    },

    watchChange(id) {
      if (externs.has(id)) {
        reloadExterns.add(id);
      }
    },

    async transform(code, id) {
      if (filter(id)) {
        try {
          const result = await opt.optimizeModule(code);
          const map = result.map;
          code = result.code;
          return map ? { code, map } : { code };
        } catch (err) {
          this.error(err.toString());
        }
      }
    },

    async renderChunk(code) {
      try {
        const result = await opt.optimizeChunk(code);
        const map = result.map;
        code = result.code;
        return map ? { code, map } : { code };
      } catch (err) {
        this.error(err.toString());
      }
    },

    async writeBundle() {
      if (propertyMap && options.renameProperties?.pattern !== void 0) {
        await fs.writeFile(propertyMap.id, opt.exportPropertyMap());
      }
    }
  };
};
