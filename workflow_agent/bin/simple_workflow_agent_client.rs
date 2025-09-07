//! A simple HTTP client example to test the A2A client.


use a2a_rs::{
    HttpClient,
    domain::{Message, Part},
    services::AsyncA2AClient,
};
use clap::{Parser};
use serde_json::Map;


use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Host to bind the server to
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "8080")]
    port: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
    #[clap(long, default_value = "./documentation/demo_workflow_management/mix_agent_tools_workflow.json")]
    graph_file: String,
    #[clap(long, default_value = "load_workflow")]
    generation_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Parse command-line arguments
    let args = Args::parse();

     /************************************************/
    /* Setting proper log level. Default is INFO    */
    /************************************************/ 
    let log_level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::WARN,
    };

    let subscriber = Registry::default()
    .with(
        // stdout layer, to view everything in the console
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(log_level))
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    /************************************************/
    /* End of Setting proper log level              */
    /************************************************/ 

 
    /************************************************/
    /* Sample A2A client either responding to       */
    /* A simple A2A Server or A2A Planner Agent     */
    /************************************************/ 

    let bind_address = format!("http://{}:{}", args.host, args.port);
    println!("Server listening on: {}", bind_address);

    let client = HttpClient::new(bind_address.to_string());



    println!("##############################################################");

    /************************************************/
    /* Third Task  (Combined Weather and Customer)  */
    /************************************************/ 

    let task_id_3 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    let message_id_3 = uuid::Uuid::new_v4().to_string();
    //let user_text="What is the current weather in Boston and What are details of Customer_id 1234 ?".to_string();
    let user_text="Prepare a nice welcome message for the company with customer_id 12345 , where you mention the weather from their location.".to_string();
    println!("\nUser_Query : {}",user_text);
    
    let generation_type = args.generation_type.as_str();

    let message_3 = if generation_type == "dynamic_generation" {
        Message::builder()
            .role(a2a_rs::domain::Role::User)
            .parts(vec![Part::Text {
                text: user_text,
                metadata: None,
            }])
            .message_id(message_id_3)
            .build()
    } else if generation_type == "high_level_plan" {
        Message::builder()
            .role(a2a_rs::domain::Role::User)
            .parts(vec![Part::Text {
                text: user_text,
                metadata: None,
            }])
            .metadata(Map::from_iter([
                ("high_level_plan".to_string(), serde_json::json!(true)),
            ]))
            .message_id(message_id_3)
            .build()
    } else {
        Message::builder()
            .role(a2a_rs::domain::Role::User)
            .parts(vec![Part::Text {
                text: user_text,
                metadata: None,
            }])
            .metadata(Map::from_iter([
                ("workflow_url".to_string(), serde_json::json!(args.graph_file)),
            ]))
            .message_id(message_id_3)
            .build()
    };
    

    // Send a task message
    println!("This is a task that needs to be sent to two different agents to be adressed...\n");
    println!("Sending message to agents to process tasks ...\n");

    let task_3 = client
        .send_task_message(&task_id_3, &message_3, None, Some(50))
        .await?;
    println!("\nGot response with status: {:?}", task_3.status.state);

    if let Some(response_message_3) = task_3.status.message {
        println!("\nAgent response:");
        for part in response_message_3.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    // Get the task again to verify it's stored
    println!("\nRetrieving task...");
    let task_3 = client.get_task(&task_id_3, None).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_3.id, task_3.status.state
    );


    /************************************************/
    /* End of Third Task                           */
    /************************************************/ 

   
    println!("##############################################################");


    Ok(())
}


    // Possibility to send metadata, and other type of content
    /*
            /// let message = Message::builder()
            ///     .role(Role::User)
            ///     .parts(vec![Part::Text {
            ///         text: "Hello, agent!".to_string(),
            ///         metadata: None,
            ///     }])
            ///     .message_id("msg-123".to_string())
            ///     .build();
            /// ```
            #[derive(Debug, Clone, Serialize, Deserialize, Builder)]
            pub struct Message {
                pub role: Role,
                #[builder(default = Vec::new())]
                pub parts: Vec<Part>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub metadata: Option<Map<String, Value>>, // THIS IS THE PART WE SHOULD USE TO UPLOAD A WORKFLOW
                #[serde(skip_serializing_if = "Option::is_none", rename = "referenceTaskIds")]
                pub reference_task_ids: Option<Vec<String>>,
                #[serde(rename = "messageId")]
                pub message_id: String,
                #[serde(skip_serializing_if = "Option::is_none", rename = "taskId")]
                pub task_id: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none", rename = "contextId")]
                pub context_id: Option<String>,
                #[builder(default = "message".to_string())]
                pub kind: String, // Always "message"
            }
     */