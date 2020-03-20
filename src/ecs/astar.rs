use num_traits::Zero;
use std::collections::{BinaryHeap, HashMap};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::hash::Hash;
use std::cmp::Ordering;


struct InvCmpHolder<K, P> {
    key: K,
    payload: P,
}

impl<K: PartialEq, P> PartialEq for InvCmpHolder<K, P> {
    fn eq(&self, other: &InvCmpHolder<K, P>) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: PartialEq, P> Eq for InvCmpHolder<K, P> {}

impl<K: PartialOrd, P> PartialOrd for InvCmpHolder<K, P> {
    fn partial_cmp(&self, other: &InvCmpHolder<K, P>) -> Option<Ordering> {
        other.key.partial_cmp(&self.key)
    }
}

impl<K: Ord, P> Ord for InvCmpHolder<K, P> {
    fn cmp(&self, other: &InvCmpHolder<K, P>) -> Ordering {
        other.key.cmp(&self.key)
    }
}

fn reverse_path<N: Eq + Hash>(mut parents: HashMap<N, N>, start: N) -> Vec<N> {
    let mut path = vec![start];
    while let Some(parent) = parents.remove(path.last().unwrap()) {
        path.push(parent);
    }
    path.into_iter().rev().collect()
}

pub fn astar<N, C, FN, IN, FH, FS>(
    start: &N,
    neighbours: FN,
    heuristic: FH,
    success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: Fn(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: Fn(&N) -> C,
    FS: Fn(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(InvCmpHolder {
        key: heuristic(start),
        payload: (Zero::zero(), start.clone()),
    });
    let mut parents: HashMap<N, (N, C)> = HashMap::new();
    while let Some(InvCmpHolder { payload: (cost, node), .. }) = to_see.pop() {
        if success(&node) {
            let parents = parents.into_iter().map(|(n, (p, _))| (n, p)).collect();
            return Some((reverse_path(parents, node), cost));
        }
        // We may have inserted a node several time into the binary heap if we found
        // a better way to access it. Ensure that we are currently dealing with the
        // best path and discard the others.
        if let Some(&(_, c)) = parents.get(&node) {
            if cost > c {
                continue;
            }
        }
        for (neighbour, move_cost) in neighbours(&node) {
            let new_cost = cost + move_cost;
            if neighbour != *start {
                let mut inserted = true;
                match parents.entry(neighbour.clone()) {
                    Vacant(e) => {
                        e.insert((node.clone(), new_cost));
                    }
                    Occupied(mut e) => {
                        if e.get().1 > new_cost {
                            e.insert((node.clone(), new_cost));
                        } else {
                            inserted = false;
                        }
                    }
                };
                if inserted {
                    let new_predicted_cost = new_cost + heuristic(&neighbour);
                    to_see.push(InvCmpHolder {
                        key: new_predicted_cost,
                        payload: (new_cost, neighbour),
                    });
                }
            }
        }
    }
    None
}
