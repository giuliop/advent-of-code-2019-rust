use advent2019::day18;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let problem = args.get(1).map(|x| x.as_str()).unwrap_or("None");
    let result = match problem {
        "18a" => day18::a(),
        "18b" => day18::b(),
        "None" => "Please supply a problem".to_string(),
        _ => "Not solved yet".to_string(),
    };

    println!("\n{}", result);
}
