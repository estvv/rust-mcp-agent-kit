// examples/test_orchestrator.rs
//
// Demonstrates:
// 1. Loading a profile from TOML
// 2. Spawning tools from profile
// 3. Multi-turn conversation loop with tool calling

use mcp_client::{Orchestrator, MockProvider, Profile};

fn main() {
    println!("=== MCP Orchestrator Test ===\n");

    // 1. Load profile by name (looks in profiles/{name}.toml)
    let profile_name = "personal";
    println!("Loading profile: {}", profile_name);
    
    let profile = match Profile::load_by_name(profile_name) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load profile: {}", e);
            eprintln!("Note: Run this example from the workspace root directory");
            std::process::exit(1);
        }
    };
    
    println!("Profile: {} - {}", profile.name(), profile.description());
    println!("Enabled tools: {:?}\n", profile.enabled_tools());

    // 2. Create orchestrator with mock provider (no LLM needed)
    // Mock will request tool call first, then return response
    let client = MockProvider::new("Based on the weather data, Paris is 15°C and partly cloudy.");
    let mut orchestrator = Orchestrator::new(client);

    // 3. Spawn tools from profile
    println!("Spawning tools...");
    for tool_name in profile.enabled_tools() {
        let binary = format!("target/debug/{}", tool_name);
        println!("  Spawning: {} ({})", tool_name, binary);
        
        match orchestrator.spawn_tool(&tool_name, &binary) {
            Ok(_) => println!("    OK"),
            Err(e) => eprintln!("    Failed: {}", e),
        }
    }

    // 4. Show available tools
    println!("\nAvailable tools:");
    for tool in orchestrator.tools() {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // 5. Test direct tool execution
    println!("\n=== Direct Tool Test ===");
    println!("Calling get_weather(city='Paris')...");
    
    match orchestrator.chat("What's the weather in Paris?") {
        Ok(response) => println!("Response: {}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\nDone.");
}