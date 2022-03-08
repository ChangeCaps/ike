use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use ike_id::RawLabel;

use crate::{ExclusiveSystemDescriptor, ParallelSystem, ScheduleError};

pub trait GraphNode {
    fn name(&self) -> &Cow<'static, str>;
    fn labels(&self) -> &[RawLabel];
    fn before(&self) -> &[RawLabel];
    fn after(&self) -> &[RawLabel];
}

impl GraphNode for ParallelSystem {
    fn name(&self) -> &Cow<'static, str> {
        &self.system.name()
    }

    fn labels(&self) -> &[RawLabel] {
        &self.labels
    }

    fn before(&self) -> &[RawLabel] {
        &self.before
    }

    fn after(&self) -> &[RawLabel] {
        &self.after
    }
}

impl GraphNode for ExclusiveSystemDescriptor {
    fn name(&self) -> &Cow<'static, str> {
        &self.system.name()
    }

    fn labels(&self) -> &[RawLabel] {
        &self.labels
    }

    fn before(&self) -> &[RawLabel] {
        &self.before
    }

    fn after(&self) -> &[RawLabel] {
        &self.after
    }
}

pub fn build_dependency_graph<T: GraphNode>(
    nodes: &[T],
) -> HashMap<usize, HashMap<usize, HashSet<&RawLabel>>> {
    let mut labels = HashMap::<&RawLabel, Vec<usize>>::new();

    for (index, node) in nodes.iter().enumerate() {
        for label in node.labels() {
            labels.entry(label).or_default().push(index);
        }
    }

    let mut graph = HashMap::new();

    for (index, node) in nodes.iter().enumerate() {
        let dependencies = graph.entry(index).or_insert_with(HashMap::default);
        for label in node.after() {
            match labels.get(label) {
                Some(new_dependencies) => {
                    for dependency in new_dependencies {
                        dependencies
                            .entry(*dependency)
                            .or_insert_with(HashSet::default)
                            .insert(label);
                    }
                }
                None => {
                    eprintln!(
                        "system '{}' wants so be after unknown label: '{:?}({})'",
                        node.name(),
                        label.type_name(),
                        label.id(),
                    );
                }
            }
        }

        for label in node.before() {
            match labels.get(label) {
                Some(dependants) => {
                    for dependant in dependants {
                        graph
                            .entry(*dependant)
                            .or_insert_with(HashMap::default)
                            .entry(index)
                            .or_insert_with(HashSet::default)
                            .insert(label);
                    }
                }
                None => {
                    eprintln!(
                        "system '{}' wants so be before unknown label: '{:?}({})'",
                        node.name(),
                        label.type_name(),
                        label.id(),
                    );
                }
            }
        }
    }

    graph
}

pub fn topological_order(
    graph: &HashMap<usize, HashMap<usize, HashSet<&RawLabel>>>,
) -> Result<Vec<usize>, ScheduleError> {
    fn check_cycles_and_visit(
        node: &usize,
        graph: &HashMap<usize, HashMap<usize, HashSet<&RawLabel>>>,
        sorted: &mut Vec<usize>,
        unvisited: &mut HashSet<usize>,
        current: &mut Vec<usize>,
    ) -> bool {
        if current.contains(node) {
            return true;
        } else if !unvisited.remove(node) {
            return false;
        }
        current.push(*node);
        for dependency in graph.get(node).unwrap().keys() {
            if check_cycles_and_visit(dependency, graph, sorted, unvisited, current) {
                return true;
            }
        }
        sorted.push(*node);
        current.pop();

        false
    }

    let mut sorted = Vec::with_capacity(graph.len());
    let mut current = Vec::with_capacity(graph.len());
    let mut unvisited = HashSet::with_capacity(graph.len());
    unvisited.extend(graph.keys().cloned());

    while let Some(node) = unvisited.iter().next().cloned() {
        if check_cycles_and_visit(&node, graph, &mut sorted, &mut unvisited, &mut current) {
            let mut cycle = Vec::new();

            let last_window = [*current.last().unwrap(), current[0]];

            let mut windows = current
                .windows(2)
                .chain(std::iter::once(&last_window as &[usize]));

            while let Some(&[dependant, dependency]) = windows.next() {
                cycle.push((
                    dependant,
                    graph[&dependant][&dependency]
                        .iter()
                        .map(|&label| label.clone())
                        .collect::<HashSet<_>>(),
                ));
            }

            return Err(ScheduleError::GraphCycles(cycle));
        }
    }

    Ok(sorted)
}
