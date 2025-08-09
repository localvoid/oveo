import type { HookFilter, RolldownPlugin } from "rolldown";
import { Optimizer, type OptimizerOptions } from "@oveo/optimizer";

export interface PluginOptions extends OptimizerOptions {
  readonly filter?: HookFilter,
  readonly externs?: { inlineConstValues?: boolean, import?: string[]; },
  readonly renameProperties?: { pattern?: string, map?: string; },
}

export function oveo(options: PluginOptions = {}): RolldownPlugin {
  let opt: Optimizer;
  let propertyMapData: Uint8Array | undefined;
  return {
    name: "oveo:optimizer",

    async buildStart() {
      opt = new Optimizer(options);

      const propertyMap = options.renameProperties?.map;
      if (propertyMap) {
        this.addWatchFile(propertyMap);
        try {
          propertyMapData = await this.fs.readFile(propertyMap);
          try {
            opt.importPropertyMap(propertyMapData);
          } catch (err) {
            this.warn(`Invalid property map file '${propertyMap}': ${err}`);
          }
        } catch (err) {
          // Report warnings only when minified property generation is disabled.
          if (!options.renameProperties.pattern) {
            this.warn(`Unable to read property map file '${propertyMap}': ${err}`);
          }
        }
      }

      const importExterns = options.externs?.import;
      if (importExterns) {
        for (const extern of importExterns) {
          const resolved = await this.resolve(extern);
          if (resolved) {
            this.addWatchFile(resolved.id);
            try {
              const data = await this.fs.readFile(resolved.id);
              opt.importExterns(data);
            } catch (err) {
              this.warn(`Unable to import extern file '${extern}': ${err}`);
            }
          } else {
            this.warn(`Unable to find extern file '${extern}'`);
          }
        }
      }
    },

    transform: {
      filter: options.filter ?? {
        moduleType: ["js", "jsx", "ts", "tsx"],
      },
      async handler(code, id, { moduleType }) {
        try {
          const result = await opt.transform(code, moduleType);
          const map = result.map;
          code = result.code;
          return map ? { code, map } : { code };
        } catch (err) {
          this.error(`Unable to transform module '${id}': ${err}`);
        }
      },
    },

    renderChunk: {
      async handler(code) {
        try {
          const result = await opt.renderChunk(code);
          const map = result.map;
          code = result.code;
          return map ? { code, map } : { code };
        } catch (err) {
          this.error(`Unable to optimize chunk file: ${err}`);
        }
      },
    },

    async writeBundle() {
      const propertyMap = options.renameProperties?.map;
      // If minified names are generated dynamically
      if (propertyMap && options.renameProperties?.pattern !== void 0) {
        const newData = opt.updatePropertyMap();
        if (newData) {
          try {
            await this.fs.writeFile(propertyMap, newData);
          } catch (err) {
            this.warn(`Unable to update property map file '${propertyMap}': ${err}`);
          }
        }
      }
    },
  };
};
