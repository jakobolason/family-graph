use crate::secrets::{COMMON_ANCESTOR1, COMMON_ANCESTOR1_LASTNAME, COMMON_ANCESTOR1_LIFE,
                     COMMON_ANCESTOR2, COMMON_ANCESTOR2_LASTNAME, COMMON_ANCESTOR2_LIFE};

use std::path::Path;
use calamine::{open_workbook, DataType, Reader, Xls};
use petgraph::{Directed, Direction, Graph};
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_wasm_bindgen;

// Import console.log for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    generation: i8,
    name: String,
    birthdate: String,
    last_name: String,
    address: String,
    city: String,
    landline: String,
    mobile_number: String,
    email: String,
}

impl Person {
    fn new(info: Vec<String>, generation: i8) -> Result<Self, &'static str> {
        let [
        name,
        birthdate,
        last_name,
        address,
        city,
        landline,
        mobile_number,
        email,
        ]: [String; 8] = info.try_into().map_err(|_| "Expected exactly 8 elements")?;

        Ok(Person {
            generation,
            name,
            birthdate,
            last_name,
            address,
            city,
            landline,
            mobile_number,
            email,
        })
    }

    fn default() -> Self {
        Person {
            generation: 0,
            name: "insert_name".to_string(),
            birthdate: "insert birthdate".to_string(),
            last_name: "insert_lastname".to_string(),
            address: "address here".to_string(),
            city: "city here".to_string(),
            landline: "landline here".to_string(),
            mobile_number: "mobile number here".to_string(),
            email: "email here".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct NetworkNode {
    pub id: String,
    pub label: String,
    pub title: String,
    pub level: i32,
    pub color: NodeColor,
    pub font: FontStyle,
    pub person_data: Person,
}

#[derive(Serialize, Debug)]
pub struct NodeColor {
    pub background: String,
    pub border: String,
}

#[derive(Serialize, Debug)]
pub struct FontStyle {
    pub size: i32,
    pub color: String,
}

#[derive(Serialize, Debug)]
pub struct NetworkEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub color: String,
    pub dashes: bool,
    pub width: i32,
    pub arrows: String,
}

#[derive(Serialize, Debug)]
pub struct FamilyNetworkData {
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<NetworkEdge>,
}

#[derive(Debug, PartialEq)]
enum Relationship {
    Child,
    Relative,
    Married,
    Divorced,
    Dating,
    ChildFromPartner,
    NotFound,
}
// To define the type of graph I'm using
type FamilyGraph = Graph<Person, Relationship, Directed>;

#[wasm_bindgen]
pub struct FamilyTreeProcessor {
    graph: FamilyGraph,
    node_map: HashMap<String, NodeIndex>,
}

#[wasm_bindgen]
impl FamilyTreeProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FamilyTreeProcessor {
        console_log!("Initializing FamilyTreeProcessor");
        FamilyTreeProcessor {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get_network_data(&self) -> JsValue {
        console_log!("Converting graph to network data");

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Convert nodes
        for node_index in self.graph.node_indices() {
            let person = &self.graph[node_index];
            let node_id = format!("node_{}", node_index.index());

            nodes.push(NetworkNode {
                id: node_id,
                label: person.name.clone(),
                title: format!("Click for details about {}", person.name), // Tooltip
                level: person.generation as i32,
                color: NodeColor {
                    background: self.get_generation_color(person.generation),
                    border: "#2B7CE9".to_string(),
                },
                font: FontStyle {
                    size: 16,
                    color: "#343434".to_string(),
                },
                person_data: person.clone(),
            });
        }

        // Convert edges
        for edge_index in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge_index) {
                let relationship = &self.graph[edge_index];

                edges.push(NetworkEdge {
                    from: format!("node_{}", source.index()),
                    to: format!("node_{}", target.index()),
                    label: self.relationship_label(relationship),
                    color: self.relationship_color(relationship),
                    dashes: matches!(relationship, Relationship::Divorced | Relationship::Dating),
                    width: self.relationship_width(relationship),
                    arrows: "to".to_string(),
                });
            }
        }

        let network_data = FamilyNetworkData { nodes, edges };
        serde_wasm_bindgen::to_value(&network_data).unwrap()
    }

    // Get detailed info for a specific person (called when node is clicked)
    #[wasm_bindgen]
    pub fn get_person_details(&self, node_id: &str) -> JsValue {
        console_log!("Getting details for: {}", node_id);

        // Extract node index from node_id
        if let Some(index_str) = node_id.strip_prefix("node_") {
            if let Ok(index) = index_str.parse::<usize>() {
                if let Some(node_index) = petgraph::graph::NodeIndex::new(index).into() {
                    if let Some(person) = self.graph.node_weight(node_index) {
                        return serde_wasm_bindgen::to_value(person).unwrap();
                    }
                }
            }
        }

        JsValue::NULL
    }

    // Helper methods
    fn get_generation_color(&self, generation: i8) -> String {
        match generation {
            -1 => "#fffccb".to_string(),
            0 => "#ffcccb".to_string(), // Light red for eldest
            1 => "#add8e6".to_string(), // Light blue
            2 => "#90ee90".to_string(), // Light green
            3 => "#ffb6c1".to_string(), // Light pink
            _ => "#f0f0f0".to_string(), // Light gray
        }
    }

    fn relationship_label(&self, rel: &Relationship) -> String {
        match rel {
            Relationship::Child => "Barn".to_string(),
            Relationship::Parent => "Forælder".to_string(),
            Relationship::Married => "Gift".to_string(),
            Relationship::Divorced => "Skilt".to_string(),
            Relationship::Dating => "Kærester".to_string(),
            Relationship::Relative => "Relateret".to_string(),
            _ => "Ukendt".to_string(),
        }
    }

    fn relationship_color(&self, rel: &Relationship) -> String {
        match rel {
            Relationship::Married => "#ff0000".to_string(),
            Relationship::Divorced => "#800080".to_string(),
            Relationship::Child | Relationship::Parent => "#000000".to_string(),
            Relationship::Relative => "#808080".to_string(),
            _ => "#000000".to_string(),
        }
    }

    fn relation_check(name: String) -> Relationship {
        if name.contains("~") {
            Relationship::Married
        } else if name.contains("-/-") {
            Relationship::Divorced
        } else if name.contains("-") {
            Relationship::Dating
        } else if name.contains("- -") {
            Relationship::ChildFromPartner
        } else {
            Relationship::Relative
        }
    }

    fn insert_relative(
        family: &mut FamilyGraph,
        crnt: &mut NodeIndex,
        parent: &mut NodeIndex,
        level: i8,
        new_gen: i8,
        person: Person )
    {
        let n = level - new_gen;
        let new_node = family.add_node(person);
        if n == 0 || n == -1 { // look in algorithm_ideas.md for explanation
            if n == -1 { *parent = *crnt; } // 1 edge from parent to crnt
        } else if new_gen == level {
            // siblings
            *crnt = new_node;
            return
        } else if new_gen > 0 && n > 0 {
            // went from child to it's parent(or grandparent),
            // so should look up the tree
            for _ in 0..n{
                if let Some(grandparent) = family
                    .neighbors_directed(*parent, Direction::Incoming)
                    .next()
                {
                    *parent = grandparent;
                }
            }
        }
        *crnt = new_node;
        family.add_edge(*parent, *crnt, Relationship::Child);
    }

    fn run(self, path: &Path) -> () {
        let mut workbook: Xls<_> = open_workbook(path).expect("Cannot open file");

        // Read whole worksheet data and provide some statistics
        let range = workbook.worksheet_range("Ark1")
            .expect("Cannot get worksheet");
        let total_cells = range.get_size().0 * range.get_size().1;
        let non_empty_cells: usize = range.used_cells().count();
        println!(
            "Found {} cells in 'Sheet1', including {} non empty cells",
            total_cells, non_empty_cells
        );

        let all_rows: Vec<_> = range.rows().collect();

        let entries: Vec<Vec<_>> = match all_rows.len() {
            len if len > 5 => {
                all_rows[2..len-3]
                    .split(|r| r.get(0).map_or(true, |cell| cell.is_empty()))
                    .filter(|group| !group.is_empty())
                    .map(|group| group.to_vec())
                    .collect()
            },
            _ => {
                println!("Warning: Not enough rows to trim (need >5, got {})", all_rows.len());
                Vec::new()
            }
        };
        println!("DONE, nr of entries: {}", entries.len());
        println!("{:?}", entries[3]);
        // -1 to indicate the common ancestor node, and to comply with Excel sheet standard
        let mut level: i8 = -1;

        let mut family = FamilyGraph::new();
        // Adds the common ancestors at the top
        let mut parent = family.add_node(Person {
            generation: -1,
            name: COMMON_ANCESTOR1.to_string(),
            birthdate: COMMON_ANCESTOR1_LIFE.to_string(),
            last_name: COMMON_ANCESTOR1_LASTNAME.to_string(),
            address: "".to_string(),
            city: "".to_string(),
            landline: "".to_string(),
            mobile_number: "".to_string(),
            email: "".to_string(),
        });
        let parent_partner = family.add_node(Person {
            generation: -1,
            name: COMMON_ANCESTOR2.to_string(),
            birthdate: COMMON_ANCESTOR2_LIFE.to_string(),
            last_name: COMMON_ANCESTOR2_LASTNAME.to_string(),
            address: "".to_string(),
            city: "".to_string(),
            landline: "".to_string(),
            mobile_number: "".to_string(),
            email: "".to_string(),
        });

        family.add_edge(parent, parent_partner, Relationship::Married);
        let mut crnt = parent;

        for family_group in entries {
            for person in family_group {
                // map Data into vector
                let person_vec: Vec<String> = person.iter()
                    .map(|cell| cell.to_string()).collect();
                // get the current gen from the name (amount of *)
                let name = person_vec[0].clone();
                let new_gen = name.matches("*").count() as i8;
                // Need to check if this is because person is gen. 0 or related some other way
                let relation = if new_gen == 0 {
                    FamilyTreeProcessor::relation_check(name.to_string())
                } else {
                    Relationship::Relative
                };
                let row_info: Person = Person::new(person_vec, new_gen)
                    .expect("Cannot create person from row");
                if relation == Relationship::Relative {
                    // updates crnt and parent, and inserts child into family
                    FamilyTreeProcessor::insert_relative(&mut family, &mut crnt, &mut parent, level, new_gen, row_info);
                    // update level
                    level = new_gen;
                } else if relation == Relationship::ChildFromPartner {
                    // don't mutate crnt
                    let child = family.add_node(row_info);
                    family.add_edge(crnt, child, Relationship::ChildFromPartner);
                } else {
                    // The others are for relationships in varying degrees
                    let relational = family.add_node(row_info);
                    family.add_edge(crnt, relational, relation);
                }
            }
        }
        self.graph = family;
        // let fancy_dot = Dot::with_attr_getters(
        //     &family,
        //     // Global graph attributes
        //     &[],
        //     // Edge attribute getter
        //     &|_graph, edge_ref| {
        //         // Get the edge weight (relationship type)
        //         match edge_ref.weight() {
        //             Relationship::Child => "style=solid, color=black, penwidth=2".to_owned(),
        //             Relationship::Parent => "style=solid, color=blue, penwidth=2".to_owned(),
        //             Relationship::Married => "style=bold, color=red, penwidth=3".to_owned(),
        //             Relationship::Divorced => "style=dashed, color=red, penwidth=2".to_owned(),
        //             Relationship::Dating => "style=dotted, color=pink, penwidth=2".to_owned(),
        //             Relationship::ChildFromPartner => "style=dashed, color=orange, penwidth=2".to_owned(),
        //             Relationship::Relative => "style=dashed, color=gray, penwidth=1".to_owned(),
        //             Relationship::NotFound => "style=dotted, color=lightgray, penwidth=1".to_owned(),
        //         }
        //     },
        //     // Node attribute getter
        //     &|_graph, node_ref| {
        //         let person = node_ref.1;  // Get the Person data
        //         format!("label=\"{}\", shape=box, style=filled, fillcolor=lightblue",
        //                 person.name.replace("\"", "\\\""))  // Escape quotes in names
        //     },
        // );
        // println!("Enhanced DOT format:\n{:?}", fancy_dot);
    }

    fn relationship_width(&self, rel: &Relationship) -> i32 {
        match rel {
            Relationship::Married => 3,
            Relationship::Child | Relationship::Parent => 2,
            _ => 1,
        }
}
    }

// Initialize WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM module loaded!");
}