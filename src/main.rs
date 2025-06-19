mod secrets;

use calamine::XlsxError::RelationshipNotFound;
use calamine::{Data, DataType, Error, RangeDeserializerBuilder, Reader, Xls, open_workbook};
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Direction, Graph};
use std::path::Path;
use petgraph::adj::Neighbors;
use petgraph::dot::Dot;



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

#[derive(Debug, PartialEq)]
enum Relationship {
    Parent,
    Child,
    Relative,
    Married,
    Divorced,
    Dating,
    Engaged,
    ChildFromPartner,
    NotFound,
}
// To define the type of graph I'm using
type FamilyGraph = Graph<Person, Relationship, Directed>;

fn relation_check(name: String) -> Relationship {
    if name.contains("~") {
        Relationship::Engaged
    } else if name.contains("-") {
        Relationship::Dating
    } else if name.contains("-/-") {
        Relationship::Divorced
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
    n: i8 )
{
    let new_node = family.add_node(person);
    if level == -1 || new_gen == 0 {
        // beginning of loop
        *parent = new_node;
    } else if n == 0 || n == -1 { // look in algorithm_ideas.md for explanation
        if n == -1 { *parent = *crnt; } // 1 edge from parent to crnt
    } else if new_gen == level {
        // siblings
    } else if new_gen > 0 && n > 0 {
        // went from child to it's parent(or grandparent),
        // so should look up the tree
        for _ in 0..n{
            if let Some(grandparent) = family
                .neighbors_directed(*crnt, Direction::Incoming)
                .next()
            {
                *parent = grandparent;
            }
        }
        *crnt = new_node;
        family.add_edge(*parent, *crnt, Relationship::Child);
    }
}

fn run(path: &Path) -> () {
    // opens a new workbook
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
    // alternatively, we can manually filter rows
    assert_eq!(
        non_empty_cells,
        range
            .rows()
            .flat_map(|r| r.iter().filter(|&c| c != &Data::Empty))
            .count()
    );
    let rows = range.get_size().0;
    // range.used_cells().for_each(|c| {let char = c.2; print!("{}\n", char);})
    let all_rows: Vec<_> = range.rows().collect();
    let families: Vec<Vec<_>> = all_rows
        .split(|r| r.get(0).map_or(true, |cell| cell.is_empty()))
        .filter(|group| !group.is_empty())
        .map(|group| group.to_vec())
        .collect();
    println!("DONE, nr of families: {}", families.len());
    println!("{:?}", families[3]);
    // println!("{:?}", families[2]);
    // -1 to indicate the common ancestor node
    let mut level: i8 = -1;

    let mut family = FamilyGraph::new();
    let mut parent = family.add_node(Person::default());
    let mut crnt = family.add_node(Person::default());

    for family_group in families {
        for person in family_group {
            // first entry is the direct relative
            let person_vec: Vec<String> = person.iter()
                .map(|cell| cell.to_string()).collect();
            // get the current gen from the name (amount of *)
            let name = person_vec[0].clone();
            let new_gen = name.matches("*").count() as i8;
            // Need to check if this is because person is gen. 0 or related some other way
            let relation = if new_gen == 0 {
                relation_check(name.to_string())
            } else {
                Relationship::Relative
            };
            let row_info: Person =
                Person::new(person_vec, new_gen).expect("Cannot create person from row");
            if relation == Relationship::Relative {
                let n = level - new_gen;
                // updates crnt and parent, and inserts child into family
                insert_relative(&mut family, &mut crnt, &mut parent, level, new_gen, row_info, n);
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
                Relationship::Parent => "style=solid, color=blue, penwidth=2".to_owned(),
                Relationship::Married => "style=bold, color=red, penwidth=3".to_owned(),
                Relationship::Divorced => "style=dashed, color=red, penwidth=2".to_owned(),
                Relationship::Dating => "style=dotted, color=pink, penwidth=2".to_owned(),
                Relationship::Engaged => "style=solid, color=purple, penwidth=2".to_owned(),
                Relationship::ChildFromPartner => "style=dashed, color=orange, penwidth=2".to_owned(),
                Relationship::Relative => "style=dashed, color=gray, penwidth=1".to_owned(),
                Relationship::NotFound => "style=dotted, color=lightgray, penwidth=1".to_owned(),
            }
        },
        // Node attribute getter
        &|_graph, node_ref| {
            let person = node_ref.1;  // Get the Person data
            format!("label=\"{}\", shape=box, style=filled, fillcolor=lightblue",
                    person.name.replace("\"", "\\\""))  // Escape quotes in names
        },
    );
    println!("Enhanced DOT format:\n{:?}", fancy_dot);
}

fn main() {
    let path = "./src/Wistoft_familien.xls";
    let full_path = Path::new(&path);
    if full_path.exists() {
        println!("✓ File exists at the specified path");
        run(full_path);
    } else {
        println!("✗ File does NOT exist at the specified path");

        // If it's a relative path, show what the absolute path would be
        if let Ok(absolute_path) = Path::new(&path).canonicalize() {
            println!("Absolute path would be: {}", absolute_path.display());
        } else {
            println!("Cannot determine absolute path (file doesn't exist)");
        }
    }
}
