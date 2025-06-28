//! A simple HTTP client example
//! The same logic needs to be implemented in planner agent

use a2a_rs::{
    HttpClient,
    domain::{Message, Part},
    services::AsyncA2AClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client connected to our example server
    let client = HttpClient::new("http://localhost:8080".to_string());

    // Generate a task ID
    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    println!("Created task with ID: {}", task_id);

    // Create a message
    //let message = Message::user_text("Hello, A2A agent! How are you today?".to_string());
    let message_id = uuid::Uuid::new_v4().to_string();
    let message = Message::user_text(
        "What is the weather like in Boston ?".to_string(),
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

    // Add a small delay to allow the server to process the task
    //tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Get the task again to verify it's stored
    println!("Retrieving task...");
    let task_retrieved = client.get_task(&task_id, Some(10)).await?;
    println!(
        "Retrieved task with ID: {} and state: {:?}",
        task_retrieved.id, task_retrieved.status.state
    );

    let task_id_2 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    //let message = Message::user_text("Hello, A2A agent! How are you today?".to_string());
    let message_id_2 = uuid::Uuid::new_v4().to_string();
    //let message_2 = Message::user_text("What are details of Customer 1234 ?".to_string(),message_id_2);
    let message_2 = Message::user_text("What are the benefits of rust ?".to_string(), message_id_2);

    // Send a task message
    println!("Sending message to task...");

    let task_2 = client
        .send_task_message(&task_id_2, &message_2, None, Some(50))
        .await?;
    println!("Got response with status: {:?}", task.status.state);

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


    Ok(())
}
