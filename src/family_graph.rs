use calamine::Data;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Direction, Graph};
use std::ops::{Deref, DerefMut};
use std::{
    env,
    fmt::{self, Formatter},
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Person {
    pub generation: i8,
    pub name: String,
    pub birthdate: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub landline: String,
    pub mobile_number: String,
    pub email: String,
}

impl Person {
    pub fn new(info: Vec<String>, generation: i8) -> Result<Self, &'static str> {
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
        // let new_name = format!("{}, {}", name, generation);

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
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

// Setup for making d3 json object
#[derive(Debug, serde::Serialize)]
pub struct D3Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _children: Option<Vec<D3Node>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<D3Node>,
    #[serde(flatten)]
    pub person: Person,
}

#[derive(Debug, PartialEq)]
pub enum Relationship {
    Child,
    Relative,
    Married,
    Divorced,
    Dating,
    ChildFromPartner,
}

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relationship::Child => write!(f, "Child"),
            Relationship::Relative => write!(f, "Relative"),
            Relationship::Married => write!(f, "Married"),
            Relationship::Divorced => write!(f, "Divorced"),
            Relationship::Dating => write!(f, "Kærester"),
            Relationship::ChildFromPartner => write!(f, "ChildFromPartner"),
        }
    }
}

/// Explains the relations possible in the document
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

/// This struct wraps the graph from petgraph.  
pub struct FamilyGraph(pub Graph<Person, Relationship, Directed>);

impl FamilyGraph {
    pub fn new() -> Self {
        Self(Graph::new())
    }

    // This fn describes what information is taken from the edges.
    pub fn build_subtree(&self, node_idx: NodeIndex) -> D3Node {
        let children: Vec<D3Node> = self
            .edges_directed(node_idx, Direction::Outgoing)
            // TODO: All edges should be allowed
            // filters out all partners
            .filter(|edge| matches!(edge.weight(), Relationship::Child))
            .map(|edge| {
                let child_idx = edge.target();
                self.build_subtree(child_idx)
            })
            .collect();
        D3Node {
            _children: None,
            children,
            person: self[node_idx].clone(),
        }
    }
    /// Business logic for inserting relatives into the graph
    fn insert_relative(
        &mut self,
        crnt: &mut NodeIndex,
        parent: &mut NodeIndex,
        level: i8,
        new_gen: i8,
        person: Person,
    ) {
        let n = level - new_gen;
        let new_node = self.add_node(person);
        if n == 0 || n == -1 {
            // look in algorithm_ideas.md for explanation
            if n == -1 {
                *parent = *crnt;
            } // 1 edge from parent to crnt
        } else if new_gen == level {
            // siblings
            *crnt = new_node;
            return;
        } else if n > 0 {
            // went from child to it's parent(or grandparent),
            // so should look up the tree
            for _ in 0..n {
                // Try and get the grandparent from parent
                if let Some(grandparent) =
                    self.neighbors_directed(*parent, Direction::Incoming).next()
                {
                    *parent = grandparent;
                    println!("Updated parent to grandparent: {:?}", grandparent);
                } else {
                    eprintln!("No grandparent found for node {:?}", parent);
                }
            }
        }
        *crnt = new_node;
        self.add_edge(*parent, *crnt, Relationship::Child);
    }

    /// Helper function: Sets the common relatives for the graph
    fn set_common_relatives() -> (Person, Person) {
        match env::var("COMMON_ANCESTOR1") {
            Ok(value) => println!("Common ancestor: {}", value),
            Err(e) => {
                eprintln!("Error reading COMMON_ANCESTOR1: {}", e);
                eprintln!("Make sure your .env file exists and dotenv().ok() is called");
            }
        }
        let common_ancestor1 = env::var("COMMON_ANCESTOR1").expect("COMMON_ANCESTOR1 must be set");
        let common_ancestor1_life =
            env::var("COMMON_ANCESTOR1_LIFE").expect("COMMON_ANCESTOR1_LIFE must be set");
        let common_ancestor1_lastname =
            env::var("COMMON_ANCESTOR1_LASTNAME").expect("COMMON_ANCESTOR1_LASTNAME must be set");
        let common_ancestor2 = env::var("COMMON_ANCESTOR2").expect("COMMON_ANCESTOR2 must be set");
        let common_ancestor2_life =
            env::var("COMMON_ANCESTOR2_LIFE").expect("COMMON_ANCESTOR2_LIFE must be set");
        let common_ancestor2_lastname =
            env::var("COMMON_ANCESTOR2_LASTNAME").expect("COMMON_ANCESTOR2_LASTNAME must be set");

        // Adds the common ancestors at the top
        (
            Person {
                generation: -1,
                name: common_ancestor1.to_string(),
                birthdate: common_ancestor1_life.to_string(),
                last_name: common_ancestor1_lastname.to_string(),
                address: "".to_string(),
                city: "".to_string(),
                landline: "".to_string(),
                mobile_number: "".to_string(),
                email: "".to_string(),
            },
            Person {
                generation: -1,
                name: common_ancestor2.to_string(),
                birthdate: common_ancestor2_life.to_string(),
                last_name: common_ancestor2_lastname.to_string(),
                address: "".to_string(),
                city: "".to_string(),
                landline: "".to_string(),
                mobile_number: "".to_string(),
                email: "".to_string(),
            },
        )
    }

    fn new_with_common_relatives() -> Self {
        let mut family = FamilyGraph::new();

        let (ancestor, wife_ancestor): (Person, Person) = FamilyGraph::set_common_relatives();
        let parent = family.add_node(ancestor);
        let parent_partner = family.add_node(wife_ancestor);
        family.add_edge(parent, parent_partner, Relationship::Married);
        family
    }

    /// Uses row, cols from parameters and goes through document.
    pub fn create_family(entries: Vec<Vec<&[Data]>>) -> Self {
        let mut family = FamilyGraph::new_with_common_relatives();
        // NOTE: This is 0, because the first inserted is the forefather
        let mut parent = NodeIndex::new(0);
        let mut crnt = parent;

        // -1 to indicate the common ancestor node, and to comply with Excel sheet standard
        let mut level: i8 = -1;
        for family_group in entries {
            for person in family_group {
                // map Data into vector
                let person_vec: Vec<String> = person.iter().map(|cell| cell.to_string()).collect();
                // get the current gen from the name (amount of *)
                let name = person_vec[0].clone();
                if name.to_lowercase() == "navn" {
                    continue;
                }
                let new_gen = name.matches("*").count() as i8;
                // Need to check if this is because person is gen. 0 or related some other way
                let relation = if new_gen == 0 {
                    relation_check(name)
                } else {
                    Relationship::Relative
                };
                let row_info: Person =
                    Person::new(person_vec, new_gen).expect("Cannot create person from row");
                if relation == Relationship::Relative {
                    // updates crnt and parent, and inserts child into family
                    family.insert_relative(&mut crnt, &mut parent, level, new_gen, row_info);
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
        family
    }
}

impl Default for FamilyGraph {
    fn default() -> Self {
        Self::new()
    }
}
// Use these such that i don't have to type self.0 everywhere
impl Deref for FamilyGraph {
    type Target = Graph<Person, Relationship, Directed>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FamilyGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
