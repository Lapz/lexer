use crate::analysis::{AnalysisState, Interval};
use crate::instructions::{Function, Instruction, Register, POINTER_WIDTH, STACK_POINTER};
use indexmap::{IndexMap, IndexSet};

use petgraph::{graphmap::GraphMap, Undirected};

use std::hash;
use util::symbol::Symbols;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StackLocation {
    offset: u64,
    spill:Register,
}

const MAX_REGISTER: usize = 3;
#[derive(Debug)]
pub struct Allocator<'a> {
    state: AnalysisState,
    function: &'a mut Function,
    ranges: IndexSet<Interval>,
    active: IndexMap<Register, Interval>,
    free_registers: IndexMap<Register, Interval>,
    pub(crate) symbols: &'a mut Symbols<()>,
    location: IndexMap<Register, StackLocation>,
    current_register: usize,
    offset: u64,
}

macro_rules! hashset {
    () => { IndexSet::new() };

    ( $($x:expr),* ) => {{
        let mut l = IndexSet::new();
        $(
            l.insert($x);
        )*
            l
    }};
}

macro_rules! hashmap {
    () => { IndexMap::new() };

    ( $($key:expr => $value:expr),* ) => {{
        let mut l = IndexMap::new();
        $(
            l.insert($key,$value);
        )*
            l
    }};
}

impl<'a> Allocator<'a> {
    pub fn new(symbols: &'a mut Symbols<()>, function: &'a mut Function) -> Self {
        Self {
            state: AnalysisState::new(function),
            symbols,
            function,
            active: hashmap!(),
            ranges: hashset!(),
            current_register: 10,
            free_registers: hashmap!(),
            location: hashmap!(),
            offset: 0,
        }
    }

    pub fn allocate(&mut self) {
        let mut blocks = Vec::new();

        std::mem::swap(&mut blocks, &mut self.function.blocks);

        for (id, _) in &blocks {
            let mut intervals = IndexMap::new();

            std::mem::swap(&mut intervals, &mut self.state.intervals[id]);
            for (reg, interval) in &intervals {
                self.expire(*reg, *interval);

                if self.active.len() == MAX_REGISTER {
                    self.spill_at_interval(*reg, *interval)
                } else {
                    self.free_registers.insert(Register::new(), *interval);

                    self.active.insert(*reg, *interval);

                    self.active.sort_keys();
                }
            }

            std::mem::swap(&mut self.state.intervals[id], &mut intervals);
        }

        std::mem::swap(&mut self.function.blocks, &mut blocks);

        println!("{:#?}", self.location);
    }

    fn expire(&mut self, reg: Register, interval: Interval) {
        if self.active.is_empty() {
            return;
        }

        while !self.active.is_empty() {
            let (active_reg, active_interval) = self.active.swap_remove_index(0).unwrap();

            if active_interval.end >= interval.start {
                self.active.insert(active_reg, active_interval);
                return;
            }

            self.free_registers.insert(active_reg, active_interval);
        }
    }

    fn spill_at_interval(&mut self, reg: Register, interval: Interval) {
        let (spill_register, spill_interval) = self.active.pop().unwrap();

        if spill_interval.end > interval.end {
            self.free_registers.insert(reg, spill_interval);

            self.location.insert(
                spill_register,
                StackLocation {
                    spill:reg,
                    offset: self.offset,
                },
            );

            self.offset += POINTER_WIDTH;

            self.active.remove(&spill_register);

            self.active.sort_keys();
        } else {
            self.location.insert(
                reg,
                StackLocation {
                    spill:reg,
                    offset: self.offset,
                },
            );

            self.offset += POINTER_WIDTH;
        }
    }
}
