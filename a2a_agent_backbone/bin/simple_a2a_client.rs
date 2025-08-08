//! A simple HTTP client example to test the A2A client.


use a2a_rs::{
    HttpClient,
    domain::{Message, Part},
    services::AsyncA2AClient,
};
use clap::{Parser};


use tracing::{ Level,info};
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

    /* 
    /************************************************/
    /* First Task (query related to weather)       */
    /************************************************/ 
     
    // Generate a task ID
    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    println!("Created task with ID: {}", task_id);

    // Create a message
    let message_id = uuid::Uuid::new_v4().to_string();

    let message = Message::user_text(
        "What is the current weather in Boston ?".to_string(),
        message_id,
    );


    // Send a task message
    println!("Sending message to task...");
    let task = client
        .send_task_message(&task_id, &message, None, Some(50))
        .await?;
    println!("Got response with status: {:?}", task.status.state);

    if let Some(response_message) = task.status.message {
        println!("Agent response:");
        for part in response_message.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    // Get the task again to verify it's stored
    println!("Retrieving task...");
    let task_retrieved = client.get_task(&task_id, Some(10)).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_retrieved.id, task_retrieved.status.state
    );

    /************************************************/
    /* End of First Task                            */
    /************************************************/ 

    /************************************************/
    /* Second Task (query related to customer)      */
    /************************************************/ 

    let task_id_2 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    let message_id_2 = uuid::Uuid::new_v4().to_string();
    let message_2 = Message::user_text("What are details of Customer_id 1234 ?".to_string(),message_id_2);

    // Send a task message
    println!("Sending message to task...");

    let task_2 = client
        .send_task_message(&task_id_2, &message_2, None, Some(50))
        .await?;
    println!("Got response with status: {:?}", task_2.status.state);

    if let Some(response_message_2) = task_2.status.message {
        println!("Agent response:");
        for part in response_message_2.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    // Get the task again to verify it's stored
    println!("Retrieving task...");
    let task_2 = client.get_task(&task_id_2, None).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_2.id, task_2.status.state
    );


    /************************************************/
    /* End of Second Task                           */
    /************************************************/ 
  
     */

    /************************************************/
    /* Third Task  (Combined Weather and Customer)  */
    /************************************************/ 

    let task_id_3 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    let message_id_3 = uuid::Uuid::new_v4().to_string();
    let user_text="What is the current weather in Boston and What are details of Customer_id 1234 ?".to_string();
    println!("\nUser_Query : {}",user_text);
    let message_3 = Message::user_text(user_text, message_id_3);

    // Send a task message
    println!("This is a task that needs to be sent to two different agents to be adressed...\n");
    println!("Sending message to agents to process tasks ...");

    let task_3 = client
        .send_task_message(&task_id_3, &message_3, None, Some(50))
        .await?;
    println!("Got response with status: {:?}", task_3.status.state);

    if let Some(response_message_3) = task_3.status.message {
        println!("Agent response:");
        for part in response_message_3.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    // Get the task again to verify it's stored
    println!("Retrieving task...");
    let task_3 = client.get_task(&task_id_3, None).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_3.id, task_3.status.state
    );


    /************************************************/
    /* End of Third Task                           */
    /************************************************/ 

   
    
      
    /************************************************/
    /* Fourth Task  (Query unrelated to tools)       */
    /************************************************/ 

    let task_id_4 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    let message_id_4 = uuid::Uuid::new_v4().to_string();
    let user_text="Make a description of the benefits of rust in less than 400 words.".to_string();
    println!("\nUser_Query : {}",user_text);
    let message_4 = Message::user_text(user_text, message_id_4);

    // Send a task message
    println!("This is an general knowledge task...\n");
    println!("Sending message to agents to process tasks ...");

    let task_4 = client
        .send_task_message(&task_id_4, &message_4, None, Some(50))
        .await?;
    //info!("Got response with status: {:?}", task_4.status.state);
    //info!("Got response with message: {:?}", task_4.status.message);

    if let Some(response_message_4) = task_4.status.message {
        println!("Agent response:");
        for part in response_message_4.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    // Get the task again to verify it's stored
    println!("Retrieving task...");
    let task_4 = client.get_task(&task_id_4, None).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_4.id, task_4.status.state
    );

    /************************************************/
    /* End of Fourth Task                           */
    /************************************************/ 



    Ok(())
}
