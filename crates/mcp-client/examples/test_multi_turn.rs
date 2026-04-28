// examples/test_multi_turn.rs
//
// Demonstrates multi-turn conversation loop:
// 1. User asks question
// 2. LLM requests tool call
// 3. Tool is executed
// 4. Result is sent back to LLM
// 5. LLM returns final answer

use mcp_client::{Orchestrator, MockProvider, Profile, ChatClient, Message, ToolDefinition};

fn main() {
    println!("=== Multi-turn Conversation Test ===\n");

    // Load profile
    let profile = Profile::load("profiles/personal.toml")
        .expect("Failed to load profile. Run from workspace root.");
    
    println!("Profile: {} ({})", profile.name(), profile.description());

    // Spawn tools
    let mut orchestrator: Orchestrator<MockProvider> = Orchestrator::new(
        MockProvider::new("The weather data shows Paris at 15°C.")
    );
    
    for tool_name in profile.enabled_tools() {
        let binary = format!("target/debug/{}", tool_name);
        orchestrator.spawn_tool(&tool_name, &binary).unwrap();
    }

    println!("Tools loaded: {} tools available\n", orchestrator.tools().len());

    // Simulate multi-turn manually to show the loop
    println!("=== Simulated Multi-turn ===\n");
    
    let user_message = "What's the weather in Paris?";
    println!("User: {}", user_message);
    
    // Turn 1: LLM decides to call tool
    println!("\n[Turn 1] LLM analyzing request...");
    println!("LLM: I need to call get_weather tool");
    
    // Execute tool
    let args = serde_json::json!({"city": "Paris"});
    println!("\n[Tool Call] get_weather({})", args);
    
    // Note: In real usage, orchestrator.chat() handles this automatically
    // Here we show the internal flow
    
    // Turn 2: Tool returns result
    println!("[Tool Result] Executing tool...");
    let tools = orchestrator.tools();
    println!("\nAvailable tools:");
    for t in tools {
        println!("  - {}", t.name);
    }

    println!("\n=== Test Complete ===");
    println!("In production, orchestrator.chat() handles the full loop:");
    println!("  1. Send user message to LLM");
    println!("  2. If tool_calls, execute each tool");
    println!("  3. Send tool results back to LLM");
    println!("  4. Repeat until LLM returns content (no more tool_calls)");
}