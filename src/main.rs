use std::fs::File;

use std::collections::HashSet;

extern crate serde_json;
#[macro_use] extern crate serde_derive;
use clap::Parser;

use pombase_gocam::gocam_parse;

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

    let mut source = File::open(args.model).unwrap();
    let model = gocam_parse(&mut source).unwrap();

    let mut seen_nodes = HashSet::new();

    let edges: Vec<_> = model.facts()
        .map(|fact| {
            seen_nodes.insert(fact.subject.clone());
            seen_nodes.insert(fact.object.clone());

            CytoscapeEdge {
                data: CytoscapeEdgeData {
                    id: fact.id(),
                    label: fact.property_label.clone(),
                    source: fact.subject.clone(),
                    target: fact.object.clone(),
                }
            }
        }).collect();

    let nodes: Vec<_> = model.individuals()
        .filter_map(|individual| {
            if !seen_nodes.contains(&individual.id) {
                return None;
            }
   
            let Some(individual_type) = individual.types.get(0)
            else {
                return None;
            };

            let individual_type = individual_type.to_owned();

            let (Some(ref label), Some(ref id)) = (individual_type.label, individual_type.id)
            else {
                return None;
            };
            let label = format!("{} ({})", label, id);
            Some(CytoscapeNode {
                data: CytoscapeNodeData {
                    id: individual.id.clone(),
                    label,
                }
            })
        }).collect();

    let nodes_string = serde_json::to_string(&nodes).unwrap();
    let edges_string = serde_json::to_string(&edges).unwrap();

    println!("nodes: {},", nodes_string);
    println!("edges: {}", edges_string);
}
