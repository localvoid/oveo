import type { HookFilter, RolldownPlugin } from 'rolldown';
import { Optimizer, type OptimizerOptions } from '@oveo/optimizer';

export interface UrlOptions {
  /** Absolute base URL, must end with `'/'`. Omitted (with `url: true`) enables auto-detection from Vite `base`. */
  readonly baseURL?: string;
}

export interface PluginOptions extends Omit<OptimizerOptions, 'url'> {
  readonly filter?: HookFilter;
  readonly externs?: { inlineConstValues?: boolean; import?: string[] };
  readonly renameProperties?: { pattern?: string; map?: string };
  /** Explicit `{ baseURL }`, `true` for auto-detection from Vite `base`, or `false`/omitted to disable. */
  readonly url?: boolean | UrlOptions;
}

export function oveo(
  options: PluginOptions = {},
): RolldownPlugin & { apply?: 'build'; configResolved?: (config: unknown) => void } {
  let opt: Optimizer;
  let propertyMapData: Uint8Array | undefined;
  let viteBase: string | undefined;
  return {
    name: 'oveo:optimizer',
    apply: 'build', // Vite build-mode only

    configResolved(config: unknown) {
      const base = (config as { base?: unknown } | null | undefined)?.base;
      if (typeof base === 'string' && base.length > 0) {
        viteBase = base;
      }
    },

    async buildStart() {
      const explicit = typeof options.url === 'object' ? options.url.baseURL : undefined;
      const autoEnabled =
        options.url === true ||
        (typeof options.url === 'object' && options.url.baseURL === undefined);
      let baseURL = explicit;
      if (baseURL === undefined && autoEnabled) {
        if (viteBase === undefined) {
          this.warn(
            'oveo: `url` auto-detection needs Vite `base`, but it was not resolved; URL rewrite is disabled.',
          );
        } else if (viteBase === './' || viteBase === '') {
          this.warn(
            `oveo: ignoring unsupported Vite \`base\` ${JSON.stringify(viteBase)} for URL rewrite; URL rewrite is disabled.`,
          );
        } else {
          baseURL = viteBase;
        }
      }
      if (typeof baseURL === 'string') {
        if (baseURL.length === 0) {
          this.error('oveo: `url.baseURL` must not be empty.');
        }
        if (!baseURL.endsWith('/')) {
          this.error(`oveo: \`url.baseURL\` must end with '/' (got ${JSON.stringify(baseURL)}).`);
        }
      }
      const { url: _ignored, ...rest } = options;
      opt = new Optimizer(
        baseURL === undefined ? { ...rest, url: undefined } : { ...rest, url: { baseURL } },
      );

      const propertyMap = options.renameProperties?.map;
      if (propertyMap) {
        this.addWatchFile(propertyMap);
        try {
          propertyMapData = await this.fs.readFile(propertyMap);
          try {
            opt.importPropertyMap(propertyMapData);
          } catch (err) {
            this.warn(`Invalid property map file '${propertyMap}': ${String(err)}`);
          }
        } catch (err) {
          // Report warnings only when minified property generation is disabled.
          if (!options.renameProperties.pattern) {
            this.warn(`Unable to read property map file '${propertyMap}': ${String(err)}`);
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
              this.warn(`Unable to import extern file '${extern}': ${String(err)}`);
            }
          } else {
            this.warn(`Unable to find extern file '${extern}'`);
          }
        }
      }
    },

    transform: {
      filter: options.filter ?? {
        moduleType: ['js', 'jsx', 'ts', 'tsx'],
      },
      async handler(code, id, { moduleType }) {
        try {
          const result = await opt.transform(code, moduleType);
          const map = result.map;
          code = result.code;
          return map ? { code, map } : { code };
        } catch (err) {
          this.error(`Unable to transform module '${id}': ${String(err)}`);
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
          this.error(`Unable to optimize chunk file: ${String(err)}`);
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
            this.warn(`Unable to update property map file '${propertyMap}': ${String(err)}`);
          }
        }
      }
    },
  };
}
