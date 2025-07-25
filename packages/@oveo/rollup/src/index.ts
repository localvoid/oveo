import type { Plugin, ResolvedId } from "rollup";
import { createFilter, type FilterPattern } from "@rollup/pluginutils";
import { Optimizer, type OptimizerOptions } from "@oveo/optimizer";

export interface PluginOptions extends OptimizerOptions {
  readonly include?: FilterPattern | undefined;
  readonly exclude?: FilterPattern | undefined;
  readonly externs?: string[],
  readonly renameProperties?: { pattern?: string, map?: string; },
}

export function oveo(options: PluginOptions = {}): Plugin {
  const filter = createFilter(
    options?.include ?? /\.(m?js|m?tsx?)$/,
    options?.exclude,
  );
  const externs = new Set<string>();

  let reloadExterns = new Set<string>();
  let reloadPropertyMap = true;
  let init = false;
  let propertyMap: ResolvedId | null = null;

  if (options.externs !== void 0) {
    for (const extern of options.externs) {
      externs.add(extern);
      reloadExterns.add(extern);
    }
  }

  const opt = new Optimizer(options);
  return {
    name: "oveo:optimizer",

    async buildStart() {
      if (!init) {
        init = true;
        if (options.renameProperties?.map !== void 0) {
          propertyMap = await this.resolve(options.renameProperties.map);
          if (propertyMap) {
            this.addWatchFile(propertyMap.id);
          } else {
            this.error(`Failed to resolve property map path '${options.renameProperties.map}'"`);
          }
        }

      }
      if (propertyMap !== null) {
        if (reloadPropertyMap) {
          try {
            this.addWatchFile(propertyMap.id);
            const data = await this.fs.readFile(propertyMap.id);
            opt.importPropertyMap(data);
          } catch (err) {
            this.warn("Failed to load property map: " + err);
          }
        }
      }
      if (reloadExterns.size > 0) {
        for (const extern of reloadExterns) {
          try {
            const resolved = await this.resolve(extern);
            if (resolved) {
              this.addWatchFile(resolved.id);
              const data = await this.fs.readFile(resolved.id);
              opt.importExterns(data);
            }
          } catch (err) {
            this.error(err);
          }
        }
        reloadExterns.clear();
      }
    },

    watchChange(id) {
      if (propertyMap !== null && propertyMap.id === id) {
        reloadPropertyMap = true;
      } else if (externs.has(id)) {
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
      if (propertyMap) {
        await this.fs.writeFile(propertyMap.id, opt.exportPropertyMap());
      }
    }
  };
};
