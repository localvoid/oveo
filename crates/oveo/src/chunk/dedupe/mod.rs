use oxc_allocator::Address;
use rustc_hash::FxHashMap;

mod hash;

pub use hash::dedupe_hash;

#[derive(Default)]
pub struct DedupeState {
    pub scopes: Vec<FxHashMap<[u8; 20], Address>>,
    pub expressions: FxHashMap<Address, Option<Address>>,
    pub duplicates: u32,
}

impl DedupeState {
    pub fn add(&mut self, address: Address, hash: [u8; 20]) {
        let mut original = true;
        for scope in &mut self.scopes {
            if let Some(original_address) = scope.get(&hash) {
                self.duplicates += 1;
                self.expressions.insert(address, Some(*original_address));
                original = false;
                break;
            }
        }
        if original {
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(hash, address);
                self.expressions.insert(address, None);
            }
        }
    }
}
