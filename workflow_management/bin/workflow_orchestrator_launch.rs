use std::env;
use workflow_management::graph::config::load_graph_from_file;
use workflow_management::graph::graph_orchestrator::PlanExecutor;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_workflow_file>", args[0]);
        return;
    }

    let workflow_file = &args[1];
    println!("Loading workflow from: {}", workflow_file);

    match load_graph_from_file(workflow_file) {
        Ok(graph) => {
            println!("Workflow loaded successfully. Plan: {}", graph.id);
            let mut executor = PlanExecutor::new(graph);
            if let Err(e) = executor.execute_plan().await {
                eprintln!("Error executing plan: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error loading workflow: {}", e);
        }
    }
}
