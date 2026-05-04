// examples/test_multi_turn_real.rs
//
// Real multi-turn test with Ollama (requires ollama serve + model)

use mcp_client::{Orchestrator, OllamaProvider, SkillLoader};

fn main() {
    println!("=== Multi-turn with Ollama ===\n");

    // Check if Ollama is available
    println!("Checking Ollama...");
    let test_client = OllamaProvider::new("http://localhost:11434", "glm-5:cloud");

    let loader = SkillLoader::new();
    let manifest = match loader.load_by_name("personal") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load skill: {}. Run from workspace root.", e);
            std::process::exit(1);
        }
    };
    let skill = manifest.render(&std::collections::HashMap::new());

    println!("Skill: {}\n", skill.name());

    // Create orchestrator
    let mut orchestrator = Orchestrator::new(test_client);

    // Spawn tools
    println!("Spawning tools from skill...");
    for tool_name in skill.enabled_tools() {
        let binary = format!("target/debug/{}", tool_name);
        match orchestrator.spawn_tool(&tool_name, &binary) {
            Ok(_) => println!("  {} OK", tool_name),
            Err(e) => eprintln!("  {} FAILED: {}", tool_name, e),
        }
    }

    println!("\nTools available:");
    for tool in orchestrator.tools() {
        println!("  - {}", tool.name);
    }

    // Multi-turn chat
    println!("\n=== Chat Test ===");
    let question = "What's the weather like in Paris right now?";
    println!("User: {}\n", question);

    match orchestrator.chat(question) {
        Ok(response) => {
            println!("Assistant: {}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nMake sure Ollama is running: ollama serve");
            eprintln!("And you have a model: ollama pull glm4");
        }
    }
}
