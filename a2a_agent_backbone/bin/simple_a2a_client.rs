//! A simple HTTP client example to test the A2A client.


use a2a_rs::{
    HttpClient,
    domain::{Message, Part},
    services::AsyncA2AClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    /************************************************/
    /* Sample A2A client either responding to       */
    /* A simple A2A Server or A2A Planner Agent     */
    /************************************************/ 

    // In our config simple a2a agent respond to port 8080
    let client = HttpClient::new("http://localhost:8080".to_string());
    
    // In our config planner of planners respond to port 9080
    //let client = HttpClient::new("http://localhost:9080".to_string());

    /************************************************/
    /* First Task (query related to weather)       */
    /************************************************/ 
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
    let message_2 = Message::user_text("What are details of Customer 1234 ?".to_string(),message_id_2);

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

    /************************************************/
    /* Third Task  (Query unrelated to tools)       */
    /************************************************/ 

    let task_id_3 = format!("task-{}", uuid::Uuid::new_v4());

    // Create a message
    //let message = Message::user_text("Hello, A2A agent! How are you today?".to_string());
    let message_id_3 = uuid::Uuid::new_v4().to_string();
    //let message_2 = Message::user_text("What are details of Customer 1234 ?".to_string(),message_id_2);
    let message_3 = Message::user_text("What are the benefits of rust ?".to_string(), message_id_3);

    // Send a task message
    println!("Sending message to task...");

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

    Ok(())
}
