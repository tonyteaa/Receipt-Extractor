use rand::Rng;
use std::env;

fn generate_license_key(tier: &str, count: usize) {
    let tier_prefix = tier.to_uppercase();
    
    println!("\n🔑 Generating {} {} License Keys:\n", count, tier_prefix);
    println!("─────────────────────────────────────");
    
    for i in 1..=count {
        let key = format!(
            "{}-{}-{}-{}",
            tier_prefix,
            generate_segment(4),
            generate_segment(4),
            generate_segment(4)
        );
        
        println!("{}. {}", i, key);
    }
    
    println!("─────────────────────────────────────\n");
}

fn generate_segment(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn print_usage() {
    println!("\n📋 License Key Generator");
    println!("════════════════════════════════════════");
    println!("\nUsage:");
    println!("  cargo run --bin keygen [count]");
    println!("\nArguments:");
    println!("  [count]  - Number of keys to generate (default: 1)");
    println!("\nExamples:");
    println!("  cargo run --bin keygen");
    println!("  cargo run --bin keygen 5");
    println!("\n════════════════════════════════════════\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // All licenses are Pro tier now
    let tier = "pro";

    let count = if args.len() >= 2 {
        match args[1].parse::<usize>() {
            Ok(n) if n > 0 && n <= 100 => n,
            Ok(_) => {
                eprintln!("❌ Error: Count must be between 1 and 100");
                print_usage();
                return;
            }
            Err(_) => {
                print_usage();
                return;
            }
        }
    } else {
        1
    };

    generate_license_key(tier, count);

    println!("💡 Pro License Features:");
    println!("   • Groq + OpenAI (cloud processing)");
    println!("   • Fast (2-5s per receipt)");
    println!("   • Automatic fallback");
    println!("   • 5 device activations");

    println!();
}

