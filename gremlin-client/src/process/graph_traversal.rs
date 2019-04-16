use crate::conversion::FromGValue;
use crate::process::bytecode::Bytecode;
use crate::process::strategies::TraversalStrategies;
use crate::structure::Labels;
use crate::structure::P as Predicate;
use crate::{GValue, GremlinResult, Vertex};
use std::marker::PhantomData;

pub struct GraphTraversal<S, E: FromGValue> {
    start: PhantomData<S>,
    end: PhantomData<E>,
    strategies: TraversalStrategies,
    bytecode: Bytecode,
}

impl<S, E: FromGValue> GraphTraversal<S, E> {
    pub fn new(strategies: TraversalStrategies, bytecode: Bytecode) -> GraphTraversal<S, E> {
        GraphTraversal {
            start: PhantomData,
            end: PhantomData,
            bytecode,
            strategies,
        }
    }
    pub fn bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn has_label<L>(mut self, labels: L) -> Self
    where
        L: Into<Labels>,
    {
        self.bytecode.add_step(
            String::from("hasLabel"),
            labels.into().0.into_iter().map(GValue::from).collect(),
        );
        self
    }

    pub fn add_v<T>(mut self, label: T) -> GraphTraversal<Vertex, Vertex>
    where
        T: Into<Labels>,
    {
        self.bytecode.add_step(
            String::from("addV"),
            label.into().0.into_iter().map(GValue::from).collect(),
        );

        GraphTraversal::new(self.strategies, self.bytecode)
    }

   
    pub fn has<P>(mut self, key: &str, predicate: P) -> Self
    where
        P: Into<Predicate>,
    {
        let p = predicate.into();
        self.bytecode.add_step(
            String::from("has"),
            vec![String::from(key).into(), p.into()],
        );
        self
    }

    pub fn out<L>(mut self, labels: L) -> GraphTraversal<S, Vertex>
    where
        L: Into<Labels>,
    {
        self.bytecode.add_step(
            String::from("out"),
            labels.into().0.into_iter().map(GValue::from).collect(),
        );

        GraphTraversal::new(self.strategies, self.bytecode)
    }
    pub fn to_list(&self) -> GremlinResult<Vec<E>> {
        self.strategies.apply(self)?.collect()
    }
}
