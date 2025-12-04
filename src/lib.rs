use calamine::{DataType, Reader, Xls, open_workbook};
use dotenv::dotenv;
use graphviz_rust::attributes::packmode::graph;
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec, parse,
};
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Direction, Graph};
use std::fmt::Formatter;
use std::path::Path;
use std::{env, fmt};

#[derive(Debug)]
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
        let new_name = format!("{}, {}", name, generation);

        Ok(Person {
            generation,
            name: new_name,
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

impl fmt::Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.name)
    }
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

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relationship::Child => write!(f, "Child"),
            Relationship::Relative => write!(f, "Relative"),
            Relationship::Married => write!(f, "Married"),
            Relationship::Divorced => write!(f, "Divorced"),
            Relationship::Dating => write!(f, "Kærester"),
            Relationship::ChildFromPartner => write!(f, "ChildFromPartner"),
            Relationship::NotFound => write!(f, "NotFound"),
        }
    }
}
// To define the type of graph I'm using
type FamilyGraph = Graph<Person, Relationship, Directed>;

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
    person: Person,
) {
    let n = level - new_gen;
    let new_node = family.add_node(person);
    if n == 0 || n == -1 {
        // look in algorithm_ideas.md for explanation
        if n == -1 {
            *parent = *crnt;
        } // 1 edge from parent to crnt
    } else if new_gen == level {
        // siblings
        *crnt = new_node;
        return;
    } else if new_gen > 0 && n > 0 {
        // went from child to it's parent(or grandparent),
        // so should look up the tree
        for _ in 0..n {
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

pub fn run(path: &Path) -> std::io::Result<()> {
    let mut workbook: Xls<_> = open_workbook(path).expect("Cannot open file");

    // Read the whole worksheet data and provide some statistics
    let range = workbook
        .worksheet_range("Ark1")
        .expect("Cannot get worksheet");

    let all_rows: Vec<_> = range.rows().collect();
    let entries: Vec<Vec<_>> = match all_rows.len() {
        len if len > 5 => all_rows[2..len - 3]
            .split(|r| r.get(0).map_or(true, |cell| cell.is_empty()))
            .filter(|group| !group.is_empty())
            .map(|group| group.to_vec())
            .collect(),
        _ => {
            println!(
                "Warning: Not enough rows to trim (need >5, got {})",
                all_rows.len()
            );
            Vec::new()
        }
    };

    let mut family = FamilyGraph::new();

    let (ancestor, wife_ancestor): (Person, Person) = set_common_relatives();
    let mut parent = family.add_node(ancestor);
    let parent_partner = family.add_node(wife_ancestor);
    family.add_edge(parent, parent_partner, Relationship::Married);
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
                insert_relative(
                    &mut family,
                    &mut crnt,
                    &mut parent,
                    level,
                    new_gen,
                    row_info,
                );
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
    let fancy_dot = Dot::with_attr_getters(
        &family,
        // Global graph attributes
        &[],
        // Edge attribute getter
        &|_graph, edge_ref| {
            // Get the edge weight (relationship type)
            match edge_ref.weight() {
                Relationship::Child => "style=solid, color=black, penwidth=2".to_owned(),
                Relationship::Married => "style=bold, color=red, penwidth=3".to_owned(),
                Relationship::Divorced => "style=dashed, color=red, penwidth=2".to_owned(),
                Relationship::Dating => "style=dotted, color=pink, penwidth=2".to_owned(),
                Relationship::ChildFromPartner => {
                    "style=dashed, color=orange, penwidth=2".to_owned()
                }
                Relationship::Relative => "style=dashed, color=gray, penwidth=1".to_owned(),
                Relationship::NotFound => "style=dotted, color=lightgray, penwidth=1".to_owned(),
            }
        },
        // Node attribute getter
        &|_graph, node_ref| {
            let person = node_ref.1; // Get the Person data
            format!(
                "label=\"{}\", shape=box, style=filled, fillcolor=lightblue",
                person.name.replace("\"", "\\\"")
            ) // Escape quotes in names
        },
    );
    // println!("Enhanced DOT format:\n{:?}", fancy_dot);
    //let mut file = File::create("family_graph.dot")?;
    let dot_string = format!("{}", fancy_dot);

    // turn the .dot file into a string, and then into a .svg file
    let res = parse(&dot_string);
    match parse(&dot_string) {
        Ok(parsed_graph) => {
            // Try a different variable name
            let mut ctx = graphviz_rust::printer::PrinterContext::default();
            match exec(
                parsed_graph,
                &mut ctx,
                vec![CommandArg::Format(Format::Svg)],
            ) {
                Ok(svg_bytes) => {
                    std::fs::write("family_graph.svg", &svg_bytes)?;
                    println!("SVG generated successfully!");
                }
                Err(e) => eprintln!("Error generating SVG: {}", e),
            }
        }
        Err(e) => eprintln!("Error parsing DOT: {}", e),
    }

    Ok(())
}
