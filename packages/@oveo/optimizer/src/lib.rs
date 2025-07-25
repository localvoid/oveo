use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use oveo::PropertyMap;
use oveo::{Globals, add_default_globals, externs::ExternMap, optimize_chunk, optimize_module};

use std::sync::Arc;
use std::sync::RwLock;

#[napi]
pub struct Optimizer {
    inner: Arc<OptimizerState>,
}

struct OptimizerState {
    options: oveo::OptimizerOptions,
    globals: Globals,
    externs: RwLock<ExternMap>,
    property_map: RwLock<PropertyMap>,
}

#[napi]
pub struct OptimizerOutput {
    pub code: String,
    pub map: String,
}

#[napi(object)]
pub struct OptimizerOptions {
    pub hoist: Option<bool>,
    pub dedupe: Option<bool>,
    pub hoist_globals: Option<bool>,
    pub inline_extern_values: Option<bool>,
    pub singletons: Option<bool>,
    pub rename_properties: Option<RenamePropertiesOptions>,
}

#[napi(object)]
pub struct RenamePropertiesOptions {
    pub pattern: Option<String>,
}

#[napi]
impl Optimizer {
    #[napi(constructor)]
    pub fn new(options: Option<OptimizerOptions>) -> Result<Self> {
        let mut globals = Globals::default();
        add_default_globals(&mut globals);

        let (options, pattern) = if let Some(options) = options {
            let (rename_properties, pattern) =
                if let Some(rename_propeties) = &options.rename_properties {
                    let pattern = if let Some(str_pat) = &rename_propeties.pattern {
                        Some(
                            regex::Regex::new(str_pat)
                                .map_err(|err| napi::Error::from_reason(err.to_string()))?,
                        )
                    } else {
                        None
                    };
                    (true, pattern)
                } else {
                    (false, None)
                };
            (
                oveo::OptimizerOptions {
                    hoist: options.hoist.unwrap_or(false),
                    dedupe: options.dedupe.unwrap_or(false),
                    hoist_globals: options.hoist_globals.unwrap_or(false),
                    inline_extern_values: options.inline_extern_values.unwrap_or(false),
                    singletons: options.singletons.unwrap_or(false),
                    rename_properties,
                },
                pattern,
            )
        } else {
            (oveo::OptimizerOptions::default(), None)
        };

        Ok(Self {
            inner: Arc::new(OptimizerState {
                options,
                globals,
                externs: RwLock::new(ExternMap::new()),
                property_map: RwLock::new(PropertyMap::new(pattern)),
            }),
        })
    }

    #[napi]
    pub fn import_externs(&mut self, data: &[u8]) -> Result<()> {
        let mut externs = self.inner.externs.write().unwrap();
        externs.import_from_json(data).map_err(|err| Error::from_reason(err.to_string()))
    }

    #[napi]
    pub fn import_property_map(&mut self, data: &[u8]) -> Result<()> {
        self.inner
            .property_map
            .write()
            .unwrap()
            .import(data)
            .map_err(|err| napi::Error::from_reason(err.to_string()))?;
        Ok(())
    }

    #[napi]
    pub fn export_property_map(&mut self) -> Uint8Array {
        let map = self.inner.property_map.read().unwrap();
        map.export().into()
    }

    #[napi(ts_return_type = "Promise<OptimizerOutput>")]
    pub fn optimize_module(&self, source_text: String) -> AsyncTask<OptimizeModuleTask> {
        AsyncTask::new(OptimizeModuleTask { optimizer: self.inner.clone(), source_text })
    }

    #[napi(ts_return_type = "Promise<OptimizerOutput>")]
    pub fn optimize_chunk(&self, source_text: String) -> AsyncTask<OptimizeChunkTask> {
        AsyncTask::new(OptimizeChunkTask { optimizer: self.inner.clone(), source_text })
    }
}

pub struct OptimizeModuleTask {
    optimizer: Arc<OptimizerState>,
    source_text: String,
}

impl Task for OptimizeModuleTask {
    type Output = OptimizerOutput;
    type JsValue = OptimizerOutput;

    fn compute(&mut self) -> Result<Self::Output> {
        let externs = self.optimizer.externs.read().unwrap();
        optimize_module(&self.source_text, &self.optimizer.options, &externs)
            .map(|v| OptimizerOutput { code: v.code, map: v.map })
            .map_err(|err| Error::from_reason(err.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: OptimizerOutput) -> Result<Self::JsValue> {
        Ok(output)
    }
}

pub struct OptimizeChunkTask {
    optimizer: Arc<OptimizerState>,
    source_text: String,
}

impl Task for OptimizeChunkTask {
    type Output = OptimizerOutput;
    type JsValue = OptimizerOutput;

    fn compute(&mut self) -> Result<Self::Output> {
        let property_map = self.optimizer.property_map.read().unwrap();
        optimize_chunk(
            &self.source_text,
            &self.optimizer.options,
            &self.optimizer.globals,
            &property_map,
        )
        .map(|v| OptimizerOutput { code: v.code, map: v.map })
        .map_err(|err| Error::from_reason(err.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: OptimizerOutput) -> Result<Self::JsValue> {
        Ok(output)
    }
}
