**Memory Service**
This service should store context of a session, enabling agents to share context, and from that to better work together to fulfill user request

The context of a conversation will be represented like this

'''
{
  "conversation_id": "conv-12345",
  "log_entries": [
    {
      "role": "User",
      "content": "Hello, how can I help you today?",
      "agent_id": null
    },
    {
      "role": "Agent",
      "content": "I need to find a file named 'example.rs' in the 'codebase/swarm' directory.",
      "agent_id": "agent-discovery-service"
    },
    {
      "role": "User",
      "content": "Okay, I will look for that file.",
      "agent_id": null
    }
  ]
}
'''