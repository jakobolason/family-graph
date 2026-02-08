use calamine::{Data, DataType, Reader, Xls, open_workbook};
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec, parse,
};
use petgraph::Direction;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use regex::Regex;
use std::{fs::File, io::Write, path::Path};

pub mod family_graph;
use family_graph::{AncestorHeader, FamilyGraph};
// re-export the common types
pub use family_graph::{D3Node, Person, Relationship};
pub use petgraph::graph;

pub fn create_dotviz(family: &FamilyGraph) -> std::io::Result<()> {
    let fancy_dot = Dot::with_attr_getters(
        &family.0,
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
                    println!("SVG generated successfully!");
                    std::fs::write("family_graph.svg", &svg_bytes)?;
                }
                Err(e) => {
                    eprintln!("Error generating SVG: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing DOT: {}", e);
        }
    }
    Ok(())
}

pub fn create_d3_export(family: &FamilyGraph, export_path: &str) -> std::io::Result<Vec<D3Node>> {
    let roots: Vec<NodeIndex> = family
        .node_indices()
        .filter(|&node_idx| {
            !family
                .edges_directed(node_idx, Direction::Incoming)
                .any(|edge| matches!(edge.weight(), Relationship::Child))
        })
        .collect();

    let mut tree_data: Vec<D3Node> = Vec::new();

    for root_idx in roots {
        let node_tree = family.build_subtree(root_idx);
        tree_data.push(node_tree);
    }
    let json_string =
        serde_json::to_string_pretty(&tree_data).expect("Failed to serialize tree data to JSON");
    let js_content = format!("export const familytreeData = {};", json_string);
    let mut file = File::create(export_path)?;
    file.write_all(js_content.as_bytes())?;
    println!("D3 data generated successfully: {export_path}");

    Ok(tree_data)
}

/// Creates the family graph using a path to a .xls file, and builds the optional parts
pub fn run_grapher(path: &Path, sheet_name: &str) -> std::io::Result<FamilyGraph> {
    // TODO: Should also be able to handle xlsl files later
    let mut workbook: Xls<_> = open_workbook(path).expect("Cannot open file");

    // Read the whole worksheet data and provide some statistics
    let range = workbook
        .worksheet_range(sheet_name)
        .expect("Cannot get worksheet");

    let data_offet = 2;
    let all_rows: Vec<_> = range.rows().collect();

    let re = Regex::new(
        r"af\s+(?P<n1>.+?)\s+(?P<d1>\d{4}-\d{4})\s+og\s+(?P<n2>.+?)\s+(?P<d2>\d{4}-\d{4})",
    )
    .unwrap();
    let a1_cell = all_rows
        .first()
        .expect("Is sheet empty?")
        .first()
        .expect("No first column?");
    let header_names = match re.captures(&a1_cell.to_string()) {
        Some(caps) => Some(AncestorHeader {
            name1: caps.name("n1").map_or("", |m| m.as_str()).to_string(),
            years1: caps.name("d1").map_or("", |m| m.as_str()).to_string(),
            name2: caps.name("n2").map_or("", |m| m.as_str()).to_string(),
            years2: caps.name("d2").map_or("", |m| m.as_str()).to_string(),
        }),
        _ => {
            println!("a1 cell: {:?}", a1_cell);
            panic!("could not find names in first row")
        }
    };

    fn cell_empty(r: &&[Data]) -> bool {
        r.first().map_or_else(|| true, |cell| cell.is_empty())
    }

    let entries: Vec<Vec<_>> = match all_rows.len() {
        len if len > 5 => all_rows[data_offet..len - 3]
            .split(cell_empty)
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
    let family_graph: FamilyGraph = FamilyGraph::create_family(entries, header_names);
    Ok(family_graph)
}
