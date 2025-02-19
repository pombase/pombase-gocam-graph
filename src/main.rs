use std::fs::File;

use std::collections::{HashMap, HashSet};

extern crate serde_json;
#[macro_use] extern crate serde_derive;
use clap::Parser;

use pombase_gocam::parse;

type CytoscapeId = String;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    model: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CytoscapeNodeData {
    id: CytoscapeId,
    label: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CytoscapeEdgeData {
    id: CytoscapeId,
    label: String,
    source: CytoscapeId,
    target: CytoscapeId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CytoscapeNode {
    data: CytoscapeNodeData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CytoscapeEdge {
    data: CytoscapeEdgeData
}

fn main() {
    let args = Args::parse();

    let mut rel_names = HashMap::new();

    rel_names.insert("BFO:0000050".to_owned(), "part of".to_owned());
    rel_names.insert("BFO:0000051".to_owned(), "has part".to_owned());
    rel_names.insert("RO:0002233".to_owned(), "has input".to_owned());
    rel_names.insert("RO:0002234".to_owned(), "has output".to_owned());
    rel_names.insert("RO:0002333".to_owned(), "enabled by".to_owned());
    rel_names.insert("RO:0002413".to_owned(), "directly provides input for".to_owned());

    let mut source = File::open(args.model).unwrap();
    let model = parse(&mut source).unwrap();

    let mut seen_nodes = HashSet::new();

    let edges: Vec<_> = model.facts()
        .map(|fact| {
            seen_nodes.insert(fact.subject.clone());
            seen_nodes.insert(fact.object.clone());

            let label = rel_names.get(&fact.property).unwrap().to_owned();

            CytoscapeEdge {
                data: CytoscapeEdgeData {
                    id: fact.id(),
                    label,
                    source: fact.subject.clone(),
                    target: fact.object.clone(),
                }
            }
        }).collect();

    let nodes: Vec<_> = model.individuals()
        .filter(|individual| {
            seen_nodes.contains(&individual.id)
        })
        .map(|individual| {
            let individual_type = &individual.types[0];
            let label =
                format!("{} ({})", individual_type.label.clone(),
                        individual_type.id.clone());
            CytoscapeNode {
                data: CytoscapeNodeData {
                    id: individual.id.clone(),
                    label,
                }
            }
        }).collect();

    let nodes_string = serde_json::to_string(&nodes).unwrap();
    let edges_string = serde_json::to_string(&edges).unwrap();

    println!("nodes: {},", nodes_string);
    println!("edges: {}", edges_string);
}
