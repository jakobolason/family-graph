
use std::{
    path::Path,

};
use calamine::{open_workbook, Error, Xls, Reader, Data, RangeDeserializerBuilder, DataType};
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;

#[derive(Debug)]
struct RowInformation {
    generation: u8
    name: String,
    birthdate: String,
    last_name: String,
    address: String,
    city: String,
    landline: String,
    mobile_number: String,
    email: String,
}

impl RowInformation {
    fn new(info: Vec<String>, generation: u8) -> Result<Self, &'static str> {
        let [name, birthdate, last_name, address, city, landline, mobile_number, email]: [String; 8] =
            info.try_into().map_err(|_| "Expected exactly 8 elements")?;

        Ok(RowInformation {
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
        RowInformation {
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

#[derive(Debug)]
enum Relationship {
    Parent,
    Child,
    Married,
    Divorced,
    Dating,
    Engaged,
}


fn run(path: &Path) -> () {
    // opens a new workbook
    let mut workbook: Xls<_ > = open_workbook(path).expect("Cannot open file");

    // Read whole worksheet data and provide some statistics
    if let Ok(range) = workbook.worksheet_range("Ark1") {
        let total_cells = range.get_size().0 * range.get_size().1;
        let non_empty_cells: usize = range.used_cells().count();
        println ! ("Found {} cells in 'Sheet1', including {} non empty cells",
                   total_cells, non_empty_cells);
        // alternatively, we can manually filter rows
        assert_eq ! (non_empty_cells, range.rows()
            .flat_map( | r | r.iter().filter( | &c | c != & Data::Empty)).count());
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
        let mut level: u8 = 0;
        type FamilyGraph = Graph<Person, Relationship, Directed>;

        let mut family = FamilyGraph::new();
        
        let common_ancestor = 
        for family in families {
            for person in family {
                // first entry is the direct relative
                let person_vec: Vec<String> = person.iter()
                    .map(|cell| cell.to_string())
                    .collect();
                // get the current gen from the name (amount of *)
                let name = person_vec[0].clone();
                let new_gen = name.matches("*").count() as u8;
                let relation = if new_gen == 0 {
                    // Need to check if this is because person is gen. 0 or related some other way
                    if name.matches("~").count() > 0 {
                        
                    } else if name.matches("-").count() > 0 {
                        
                    } else if name.matches()
                }
                let row_info = RowInformation::new(person_vec, new_gen).expect("Cannot create person from row");

            }
        }
    }
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
