use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use oveo::PropertyMap;
use oveo::{externs::ExternMap, optimize_chunk, optimize_module};

use std::sync::Arc;
use std::sync::RwLock;

#[napi]
pub struct Optimizer {
    inner: Arc<OptimizerState>,
}

struct OptimizerState {
    options: oveo::OptimizerOptions,
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
    pub globals: Option<GlobalsOptions>,
    pub externs: Option<ExternsOptions>,
    pub rename_properties: Option<RenamePropertiesOptions>,
}

#[napi(object)]
pub struct GlobalsOptions {
    pub include: Option<Vec<String>>,
    pub hoist: Option<bool>,
    pub singletons: Option<bool>,
}

#[napi(object)]
pub struct ExternsOptions {
    pub inline_const_values: Option<bool>,
}

#[napi(object)]
pub struct RenamePropertiesOptions {
    pub pattern: Option<String>,
}

#[napi]
impl Optimizer {
    #[napi(constructor)]
    pub fn new(options: Option<OptimizerOptions>) -> Result<Self> {
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
                    hoist: options.hoist.unwrap_or_default(),
                    dedupe: options.dedupe.unwrap_or_default(),
                    globals: options
                        .globals
                        .as_ref()
                        .map(|v| oveo::GlobalsOptions {
                            include: options
                                .globals
                                .as_ref()
                                .and_then(|v| {
                                    v.include
                                        .as_ref()
                                        .map(|include| oveo::GlobalCategory::from(include.iter()))
                                })
                                .unwrap_or_default(),
                            hoist: v.hoist.unwrap_or_default(),
                            singletons: v.singletons.unwrap_or_default(),
                        })
                        .unwrap_or_default(),
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
    pub fn update_property_map(&mut self) -> Option<Uint8Array> {
        let map = self.inner.property_map.read().unwrap();
        if map.is_dirty() { Some(map.export().into()) } else { None }
    }

    #[napi(ts_return_type = "Promise<OptimizerOutput>")]
    pub fn transform(
        &self,
        source_text: String,
        module_type: String,
    ) -> AsyncTask<TransformModuleTask> {
        AsyncTask::new(TransformModuleTask {
            optimizer: Arc::clone(&self.inner),
            source_text,
            module_type,
        })
    }

    #[napi(ts_return_type = "Promise<OptimizerOutput>")]
    pub fn render_chunk(&self, source_text: String) -> AsyncTask<RenderChunkTask> {
        AsyncTask::new(RenderChunkTask { optimizer: Arc::clone(&self.inner), source_text })
    }
}

pub struct TransformModuleTask {
    optimizer: Arc<OptimizerState>,
    source_text: String,
    module_type: String,
}

impl Task for TransformModuleTask {
    type Output = OptimizerOutput;
    type JsValue = OptimizerOutput;

    fn compute(&mut self) -> Result<Self::Output> {
        let externs = self.optimizer.externs.read().unwrap();
        optimize_module(&self.source_text, &self.module_type, &self.optimizer.options, &externs)
            .map(|v| OptimizerOutput { code: v.code, map: v.map })
            .map_err(|err| Error::from_reason(err.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: OptimizerOutput) -> Result<Self::JsValue> {
        Ok(output)
    }
}

pub struct RenderChunkTask {
    optimizer: Arc<OptimizerState>,
    source_text: String,
}

impl Task for RenderChunkTask {
    type Output = OptimizerOutput;
    type JsValue = OptimizerOutput;

    fn compute(&mut self) -> Result<Self::Output> {
        let property_map = self.optimizer.property_map.read().unwrap();
        optimize_chunk(&self.source_text, &self.optimizer.options, &property_map)
            .map(|v| OptimizerOutput { code: v.code, map: v.map })
            .map_err(|err| Error::from_reason(err.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: OptimizerOutput) -> Result<Self::JsValue> {
        Ok(output)
    }
}
