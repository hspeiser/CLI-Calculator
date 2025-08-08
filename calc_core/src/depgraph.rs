use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    // cell index -> defined symbols
    pub defines: IndexMap<usize, Vec<String>>,
    // symbol -> cells that use it
    pub uses: IndexMap<String, Vec<usize>>,
}

impl DependencyGraph {
    pub fn new() -> Self { Self::default() }
    pub fn update_cell(&mut self, cell_idx: usize, defines: Vec<String>, uses: Vec<String>) {
        self.defines.insert(cell_idx, defines.clone());
        for u in uses { self.uses.entry(u).or_default().push(cell_idx); }
    }
    pub fn dependents_of_symbol(&self, sym: &str) -> Vec<usize> {
        self.uses.get(sym).cloned().unwrap_or_default()
    }
}

