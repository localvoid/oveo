use std::sync::Arc;

use rustc_hash::FxHashMap;
use serde::Deserialize;

pub static INTRINSICS_MODULE_NAME: &str = "oveo";

#[derive(Deserialize)]
pub struct ExternModule {
    pub exports: FxHashMap<String, ExternValue>,
}

#[derive(Default, Deserialize)]
pub struct ExternMap {
    pub modules: FxHashMap<String, Arc<ExternModule>>,
}

impl ExternMap {
    pub fn new() -> Self {
        let mut modules = FxHashMap::default();

        // Add intrinsic functions
        let mut exports = FxHashMap::default();
        add_intrinsic(&mut exports, "hoist", IntrinsicFunction::Hoist, vec![arg_hoist()]);
        add_intrinsic(&mut exports, "scope", IntrinsicFunction::Scope, vec![arg_scope()]);
        add_intrinsic(&mut exports, "dedupe", IntrinsicFunction::Dedupe, vec![]);
        add_intrinsic(&mut exports, "key", IntrinsicFunction::Key, vec![]);
        modules.insert(INTRINSICS_MODULE_NAME.to_string(), Arc::new(ExternModule { exports }));

        Self { modules }
    }

    pub fn import_from_json(&mut self, raw: &[u8]) -> Result<(), serde_json::Error> {
        let mut modules = serde_json::from_slice::<FxHashMap<String, Arc<ExternModule>>>(raw)?;
        for (k, v) in modules.drain() {
            self.modules.insert(k, v);
        }
        Ok(())
    }
}

fn arg_hoist() -> ExternFunctionArgument {
    ExternFunctionArgument { hoist: true, scope: false }
}

fn arg_scope() -> ExternFunctionArgument {
    ExternFunctionArgument { hoist: false, scope: true }
}

fn add_intrinsic(
    intrinsics: &mut FxHashMap<String, ExternValue>,
    name: &str,
    kind: IntrinsicFunction,
    arguments: Vec<ExternFunctionArgument>,
) {
    intrinsics.insert(
        name.to_string(),
        ExternValue::Function(Arc::new(ExternFunction { arguments, intrinsic: Some(kind) })),
    );
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ExternValue {
    Namespace(Arc<ExternModule>),
    Function(Arc<ExternFunction>),
    Const(Arc<ExternConst>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternFunction {
    #[serde(default)]
    pub arguments: Vec<ExternFunctionArgument>,
    #[serde(default)]
    pub intrinsic: Option<IntrinsicFunction>,
}

#[derive(Deserialize)]
pub enum IntrinsicFunction {
    Hoist,
    Scope,
    Dedupe,
    Key,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternFunctionArgument {
    #[serde(default)]
    pub hoist: bool,
    #[serde(default)]
    pub scope: bool,
}

#[derive(Deserialize)]
pub struct ExternConst {
    pub value: serde_json::Value,
}
