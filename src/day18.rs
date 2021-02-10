use std::char;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::mem::discriminant;
use Cell::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum Cell {
    Wall,
    Floor,
    Door(char),
    Key(char),
    Start,
}

impl Cell {
    fn from_char(c: char) -> Cell {
        match c {
            '#' => Wall,
            '.' => Floor,
            '@' => Start,
            'A'..='Z' => Door(c),
            'a'..='z' => Key(c),
            _ => unreachable!(),
        }
    }
}

type Map = Vec<Vec<Cell>>;
type Pos = (usize, usize);

fn keys_and_start(map: &Map) -> HashMap<Cell, Pos> {
    let mut res = HashMap::new();
    for (y, line) in map.iter().enumerate() {
        for (x, cell) in line.iter().enumerate() {
            match cell {
                Start | Key(_) => {
                    res.insert(*cell, (x, y));
                }
                _ => (),
            }
        }
    }
    res
}

fn neighbors((x, y): Pos, map: &Map) -> Vec<Pos> {
    let mut res = Vec::new();
    if x > 0 {
        res.push((x - 1, y))
    }
    if y > 0 {
        res.push((x, y - 1))
    }
    if x < map[y].len() - 1 {
        res.push((x + 1, y))
    }
    if y < map.len() - 1 {
        res.push((x, y + 1))
    }
    res.retain(|(x, y)| discriminant(&map[*y][*x]) != discriminant(&Wall));
    res
}

#[derive(Clone, Debug)]
struct Path {
    pos: Pos,
    doors: HashSet<char>,
    steps: usize,
}

impl Path {
    fn extend(&self, map: &Map) -> Vec<Self> {
        neighbors(self.pos, map)
            .iter()
            .map(|&pos| Path {
                pos,
                doors: self.doors.clone(),
                steps: self.steps + 1,
            })
            .collect()
    }
}

#[derive(Debug)]
struct PathExplorer {
    paths: HashMap<Pos, (Option<char>, Path)>,
    frontier: Vec<Path>,
}

impl PathExplorer {
    fn new_from(start: Pos) -> Self {
        let path = Path {
            pos: start,
            doors: HashSet::new(),
            steps: 0,
        };
        let mut paths = HashMap::new();
        paths.insert(start, (None, path.clone()));
        Self {
            paths,
            frontier: vec![path],
        }
    }

    fn insert(&mut self, mut path: Path, map: &Map) {
        let (x, y) = path.pos;
        let mut maybe_key = None;
        match map[y][x] {
            Key(k) => maybe_key = Some(k),
            Door(d) => {
                path.doors.insert(d.to_ascii_lowercase());
            }
            _ => (),
        }
        match self.paths.get(&path.pos) {
            Some((_, old_path)) if old_path.steps < path.steps => (),
            _ => {
                self.paths.insert(path.pos, (maybe_key, path.clone()));
                self.frontier.push(path);
            }
        }
    }

    fn pop(&mut self) -> Path {
        self.frontier.pop().unwrap()
    }

    fn done(&self) -> bool {
        self.frontier.is_empty()
    }
}

fn edges_from(start: Pos, map: &Map) -> Vec<Edge> {
    let mut explorer = PathExplorer::new_from(start);

    while !explorer.done() {
        let new_paths = explorer.pop().extend(map);
        for path in new_paths {
            explorer.insert(path, map);
        }
    }

    explorer
        .paths
        .iter()
        .filter(|(_, (key, _))| key.is_some())
        .map(|(_, (key, path))| Edge {
            to_key: key.unwrap(),
            doors: path.doors.clone(),
            steps: path.steps,
        })
        .collect()
}

fn read_input(path: &str) -> Map {
    fs::read_to_string(path)
        .expect("Error reading input file")
        .lines()
        .map(|l| l.chars().map(Cell::from_char).collect())
        .collect()
}

fn graph_from_map(map: Map) -> Graph {
    let nodes = keys_and_start(&map);
    let mut graph = HashMap::new();
    for (node, pos) in nodes.iter() {
        graph.insert(*node, edges_from(*pos, &map));
    }
    graph
}

type Graph = HashMap<Cell, Vec<Edge>>;

fn all_keys(graph: &Graph) -> HashSet<char> {
    graph
        .keys()
        .filter(|&k| discriminant(k) == discriminant(&Key('_')))
        .map(|k| match k {
            Key(k) => *k,
            _ => unreachable!(),
        })
        .collect()
}

#[derive(Debug)]
struct Edge {
    to_key: char,
    doors: HashSet<char>,
    steps: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct GraphPath {
    steps: usize,
    keys: HashSet<char>,
    pos: Cell,
}

impl GraphPath {
    fn add_edge(&self, edge: &Edge) -> Self {
        let steps = self.steps + edge.steps;
        let mut keys = self.keys.clone();
        keys.insert(edge.to_key);
        let pos = Key(edge.to_key);
        Self { steps, keys, pos }
    }
}

impl Ord for GraphPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.keys
            .len()
            .cmp(&other.keys.len())
            .then_with(|| other.steps.cmp(&self.steps))
            .then(Ordering::Greater)
    }
}

impl PartialOrd for GraphPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn serialize_keys(keyset: &HashSet<char>) -> Vec<char> {
    let mut res: Vec<char> = keyset.iter().copied().collect();
    res.sort_unstable();
    res
}

pub fn a() -> String {
    let graph = graph_from_map(read_input("../input/day18"));
    let keyset = all_keys(&graph).len();
    let mut visited = HashMap::new();

    let mut paths: BinaryHeap<GraphPath> = BinaryHeap::new();
    paths.push(GraphPath {
        steps: 0,
        keys: HashSet::new(),
        pos: Start,
    });

    let mut res: Option<usize> = None;

    while let Some(path) = paths.pop() {
        if path.keys.len() == keyset && (res.is_none() || path.steps < res.unwrap()) {
            res = Some(path.steps);
        }
        let is_candidate = (res.is_none() || res.unwrap() > path.steps)
            && match visited.get(&(path.pos, serialize_keys(&path.keys))) {
                Some(steps) if *steps <= path.steps => false,
                _ => {
                    visited.insert((path.pos, serialize_keys(&path.keys)), path.steps);
                    true
                }
            };
        if is_candidate {
            for edge in &graph[&path.pos] {
                if edge.doors.iter().all(|door| path.keys.contains(door)) {
                    paths.push(path.add_edge(&edge))
                }
            }
        }
    }
    res.unwrap().to_string()
}

pub fn b() -> String {
    "".to_string()
}
