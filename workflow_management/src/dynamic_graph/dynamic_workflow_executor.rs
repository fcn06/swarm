use hierarchical_planner::{HierarchicalPlan, HighLevelStep, SimpleActivity};
use agent_core::graph::graph_definition::{ActivityInput, ActivityType, AgentConfigInput, ToolConfigInput, TaskConfigInput, Dependency, WorkflowPlanInput};
use serde_json::Value;

pub struct DynamicWorkflowExecutor;

impl DynamicWorkflowExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// Converts a HierarchicalPlan (high-level steps with suggested SimpleActivities)
    /// into a detailed WorkflowPlanInput suitable for execution.
    pub async fn convert_plan_to_workflow(&self, hierarchical_plan: HierarchicalPlan) -> Result<WorkflowPlanInput, String> {
        let mut activities_input: Vec<ActivityInput> = Vec::new();
        let mut previous_activity_id: Option<String> = None;

        for (i, high_level_step) in hierarchical_plan.steps.into_iter().enumerate() {
            let step_number = i + 1;
            let activity_id = format!("activity_{}", step_number);

            let mut current_activity_input = ActivityInput {
                activity_type: ActivityType::DirectTaskExecution, // Default type
                id: activity_id.clone(),
                description: high_level_step.description.clone(),
                r#type: "generic_step".to_string(), // Default type, can be overridden
                agent: None,
                tools: None,
                tasks: None,
                dependencies: Vec::new(),
                expected_outcome: format!("Outcome of step {}", step_number),
            };

            if let Some(simple_activity) = high_level_step.suggested_activity {
                current_activity_input.activity_type = simple_activity.activity_type.clone();
                current_activity_input.r#type = simple_activity.r#type.unwrap_or_else(|| "generic_step".to_string());
                // Use the description from the high-level step, but the type and resource details from simple_activity
                // current_activity_input.description = simple_activity.description.clone(); // Keep original high-level description for now

                match simple_activity.activity_type {
                    ActivityType::DelegationAgent => {
                        current_activity_input.agent = Some(AgentConfigInput {
                            skill_to_use: simple_activity.skill_to_use,
                            assigned_agent_id_preference: None, // Not determined at this stage
                            agent_context: None, // Not determined at this stage
                        });
                        current_activity_input.r#type = format!("agent_{}", simple_activity.id);
                    }
                    ActivityType::DirectToolUse => {
                        current_activity_input.tools = simple_activity.tool_to_use.map(|tool_name| {
                            vec![ToolConfigInput {
                                tool_to_use: Some(tool_name),
                                tool_parameters: Value::Null, // Parameters would need dynamic generation/extraction
                            }]
                        });
                        current_activity_input.r#type = format!("tool_{}", simple_activity.id);
                    }
                    ActivityType::DirectTaskExecution => {
                        current_activity_input.tasks = simple_activity.tasks_to_use;
                        current_activity_input.r#type = format!("task_{}", simple_activity.id);
                    }
                }
            } else {
                // If no specific SimpleActivity was suggested, keep it as a generic task.
                current_activity_input.tasks = Some(vec![
                    TaskConfigInput {
                        task_to_use: Some(format!("unassigned_task_{}", step_number)),
                        task_parameters: Value::String(high_level_step.description.clone()),
                    }
                ]);
                current_activity_input.r#type = "unassigned_task".to_string();
            }

            // Add sequential dependency
            if let Some(prev_id) = previous_activity_id {
                current_activity_input.dependencies.push(Dependency {
                    source: prev_id,
                    condition: None,
                });
            }

            activities_input.push(current_activity_input);
            previous_activity_id = Some(activity_id);
        }

        Ok(WorkflowPlanInput {
            plan_name: format!("dynamic_workflow_for_{}", hierarchical_plan.user_request.replace(" ", "_").to_lowercase()),
            activities: activities_input,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hierarchical_planner::{HierarchicalPlan, HighLevelStep, SimpleActivity, SimulatedLLMClient, SimulatedEmbeddingModel, HierarchicalPlanner};
    use agent_core::graph::Activity;
    use serde_json::json;

    // Helper function to create a basic Activity for testing
    fn create_activity(
        id: &str,
        description: &str,
        activity_type: ActivityType,
        r#type: &str,
        skill_to_use: Option<&str>,
        tool_to_use: Option<&str>,
        tool_parameters: Option<serde_json::Value>,
        tasks: Option<Vec<TaskConfigInput>>,
    ) -> Activity {
        Activity {
            activity_type,
            id: id.to_string(),
            description: description.to_string(),
            r#type: Some(r#type.to_string()),
            skill_to_use: skill_to_use.map(|s| s.to_string()),
            assigned_agent_id_preference: None,
            agent_context: None,
            tool_to_use: tool_to_use.map(|s| s.to_string()),
            tool_parameters,
            tasks,
            dependencies: Vec::new(),
            expected_outcome: Some(format!("Outcome of {}", id)),
            activity_output: None,
        }
    }

    #[tokio::test]
    async fn test_convert_plan_to_workflow() {
        let llm_client = Box::new(SimulatedLLMClient);
        let embedding_model = Box::new(SimulatedEmbeddingModel);

        let available_activities = vec![
            create_activity("DataFetcher", "Fetches data from various data sources.", ActivityType::DirectToolUse, "data_tool", None, Some("DataFetcherTool"), None, None),
            create_activity("SentimentAnalyzer", "Analyzes text for sentiment and extracts key entities.", ActivityType::DirectToolUse, "nlp_tool", None, Some("SentimentAnalyzerTool"), None, None),
            create_activity("ReportGenerator", "Compiles data into a structured report.", ActivityType::DirectToolUse, "reporting_tool", None, Some("ReportGeneratorTool"), None, None),
            create_activity("EmailSenderAgent", "Sends emails to specified recipients.", ActivityType::DelegationAgent, "communication_agent", Some("email_sending_skill"), None, None, None),
        ];

        let planner = HierarchicalPlanner::new(llm_client, embedding_model, available_activities)
            .await
            .expect("Failed to create planner");

        let user_request = "Analyze customer feedback and generate a summary report, then notify the marketing team.";
        let high_level_plan = planner.generate_plan(user_request).await.expect("Failed to generate high-level plan");

        let executor = DynamicWorkflowExecutor::new();
        let workflow_plan_input = executor.convert_plan_to_workflow(high_level_plan).await.expect("Failed to convert plan to workflow");

        assert_eq!(workflow_plan_input.plan_name, "dynamic_workflow_for_analyze_customer_feedback_and_generate_a_summary_report,_then_notify_the_marketing_team.");
        assert_eq!(workflow_plan_input.activities.len(), 4);

        let activity1 = &workflow_plan_input.activities[0];
        assert_eq!(activity1.id, "activity_1");
        assert_eq!(activity1.description, "Fetch customer feedback data.");
        assert_eq!(activity1.activity_type, ActivityType::DirectToolUse);
        assert_eq!(activity1.r#type, "tool_DataFetcher");
        assert!(activity1.tools.is_some());
        assert_eq!(activity1.tools.as_ref().unwrap()[0].tool_to_use.as_ref().unwrap(), "DataFetcherTool");
        assert!(activity1.dependencies.is_empty());

        let activity2 = &workflow_plan_input.activities[1];
        assert_eq!(activity2.id, "activity_2");
        assert_eq!(activity2.description, "Analyze feedback for sentiment and key themes.");
        assert_eq!(activity2.activity_type, ActivityType::DirectToolUse);
        assert_eq!(activity2.r#type, "tool_SentimentAnalyzer");
        assert!(activity2.tools.is_some());
        assert_eq!(activity2.tools.as_ref().unwrap()[0].tool_to_use.as_ref().unwrap(), "SentimentAnalyzerTool");
        assert_eq!(activity2.dependencies.len(), 1);
        assert_eq!(activity2.dependencies[0].source, "activity_1");

        let activity3 = &workflow_plan_input.activities[2];
        assert_eq!(activity3.id, "activity_3");
        assert_eq!(activity3.description, "Generate a summary report.");
        assert_eq!(activity3.activity_type, ActivityType::DirectToolUse);
        assert_eq!(activity3.r#type, "tool_ReportGenerator");
        assert!(activity3.tools.is_some());
        assert_eq!(activity3.tools.as_ref().unwrap()[0].tool_to_use.as_ref().unwrap(), "ReportGeneratorTool");
        assert_eq!(activity3.dependencies.len(), 1);
        assert_eq!(activity3.dependencies[0].source, "activity_2");

        let activity4 = &workflow_plan_input.activities[3];
        assert_eq!(activity4.id, "activity_4");
        assert_eq!(activity4.description, "Send the report to the marketing team.");
        assert_eq!(activity4.activity_type, ActivityType::DelegationAgent);
        assert_eq!(activity4.r#type, "agent_EmailSenderAgent");
        assert!(activity4.agent.is_some());
        assert_eq!(activity4.agent.as_ref().unwrap().skill_to_use.as_ref().unwrap(), "email_sending_skill");
        assert_eq!(activity4.dependencies.len(), 1);
        assert_eq!(activity4.dependencies[0].source, "activity_3");
    }

     #[tokio::test]
    async fn test_convert_plan_with_unassigned_activity() {
        let llm_client = Box::new(SimulatedLLMClient);
        let embedding_model = Box::new(SimulatedEmbeddingModel);

        let available_activities = vec![
            create_activity("EmailSenderAgent", "Sends emails.", ActivityType::DelegationAgent, "communication_agent", Some("email_sending_skill"), None, None, None),
        ];

        let planner = HierarchicalPlanner::new(llm_client, embedding_model, available_activities)
            .await
            .expect("Failed to create planner");

        let user_request = "Process new user registrations and send a welcome email.";
        let high_level_plan = planner.generate_plan(user_request).await.expect("Failed to generate high-level plan");

        let executor = DynamicWorkflowExecutor::new();
        let workflow_plan_input = executor.convert_plan_to_workflow(high_level_plan).await.expect("Failed to convert plan to workflow");

        assert_eq!(workflow_plan_input.activities.len(), 4);

        // The first three steps should be generic tasks due to no matching activity
        let activity1 = &workflow_plan_input.activities[0];
        assert_eq!(activity1.id, "activity_1");
        assert_eq!(activity1.description, "Receive new user registration event.");
        assert_eq!(activity1.activity_type, ActivityType::DirectTaskExecution);
        assert_eq!(activity1.r#type, "unassigned_task");
        assert!(activity1.tasks.is_some());
        assert_eq!(activity1.tasks.as_ref().unwrap()[0].task_to_use.as_ref().unwrap(), "unassigned_task_1");

        let activity2 = &workflow_plan_input.activities[1];
        assert_eq!(activity2.id, "activity_2");
        assert_eq!(activity2.description, "Store user details in database.");
        assert_eq!(activity2.activity_type, ActivityType::DirectTaskExecution);
        assert_eq!(activity2.r#type, "unassigned_task");
        assert!(activity2.tasks.is_some());
        assert_eq!(activity2.tasks.as_ref().unwrap()[0].task_to_use.as_ref().unwrap(), "unassigned_task_2");
        assert_eq!(activity2.dependencies.len(), 1);
        assert_eq!(activity2.dependencies[0].source, "activity_1");

        let activity3 = &workflow_plan_input.activities[2];
        assert_eq!(activity3.id, "activity_3");
        assert_eq!(activity3.description, "Generate welcome email content.");
        assert_eq!(activity3.activity_type, ActivityType::DirectTaskExecution);
        assert_eq!(activity3.r#type, "unassigned_task");
        assert!(activity3.tasks.is_some());
        assert_eq!(activity3.tasks.as_ref().unwrap()[0].task_to_use.as_ref().unwrap(), "unassigned_task_3");
        assert_eq!(activity3.dependencies.len(), 1);
        assert_eq!(activity3.dependencies[0].source, "activity_2");

        // The last step should be assigned to EmailSenderAgent
        let activity4 = &workflow_plan_input.activities[3];
        assert_eq!(activity4.id, "activity_4");
        assert_eq!(activity4.description, "Send welcome email to user.");
        assert_eq!(activity4.activity_type, ActivityType::DelegationAgent);
        assert_eq!(activity4.r#type, "agent_EmailSenderAgent");
        assert!(activity4.agent.is_some());
        assert_eq!(activity4.agent.as_ref().unwrap().skill_to_use.as_ref().unwrap(), "email_sending_skill");
        assert_eq!(activity4.dependencies.len(), 1);
        assert_eq!(activity4.dependencies[0].source, "activity_3");
    }
}
