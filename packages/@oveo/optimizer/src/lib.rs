use napi::Env;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use oveo::Globals;
use oveo::add_default_globals;
use oveo::deserialize_property_map;
use oveo::externs::ExternMap;
use oveo::optimize_chunk;
use oveo::optimize_module;
use rustc_hash::FxHashMap;

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
    property_map: RwLock<FxHashMap<String, String>>,
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
    pub rename_properties: Option<bool>,
}

#[napi]
impl Optimizer {
    #[napi(constructor)]
    pub fn new(options: Option<OptimizerOptions>) -> Self {
        let mut globals = Globals::default();
        add_default_globals(&mut globals);
        Self {
            inner: Arc::new(OptimizerState {
                options: options.map_or_else(Default::default, |v| oveo::OptimizerOptions {
                    hoist: v.hoist.unwrap_or(false),
                    dedupe: v.dedupe.unwrap_or(false),
                    hoist_globals: v.hoist_globals.unwrap_or(false),
                    inline_extern_values: v.inline_extern_values.unwrap_or(false),
                    singletons: v.singletons.unwrap_or(false),
                    rename_properties: v.rename_properties.unwrap_or(false),
                }),
                globals,
                externs: RwLock::new(ExternMap::new()),
                property_map: RwLock::new(FxHashMap::default()),
            }),
        }
    }

    #[napi]
    pub fn import_externs(&mut self, data: &[u8]) -> Result<()> {
        let mut externs = self.inner.externs.write().unwrap();
        externs.import_from_json(data).map_err(|err| Error::from_reason(err.to_string()))
    }

    #[napi]
    pub fn import_property_map(&mut self, data: &[u8]) -> Result<()> {
        if let Ok(map) = deserialize_property_map(data) {
            let mut v = self.inner.property_map.write().unwrap();
            *v = map;
        }
        Ok(())
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
