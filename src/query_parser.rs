pub fn parse(query: String) -> Trigger {
    let mut trigger = Trigger {
        trigger: String::new(),
        args: vec![],
    };

    if query.is_empty() {
        return trigger;
    }

    let triggers = vec!["code", "float", "time"];
    let mut args: Vec<String> = query.split(' ').map(|s| s.to_string()).collect();
    if !triggers.contains(&args[0].as_str()) {
        args.insert(0, "application".to_string());
    }
    trigger.trigger = args[0].clone();
    trigger.args = args[1..].to_vec();

    return trigger;
}

pub struct Trigger {
    trigger: String, // application, code, float, time
    args: Vec<String>,
}
