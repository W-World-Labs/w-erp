use serde_json::{Value, json};

use super::client::CliClient;

pub(crate) fn print_mutation_result(c: &CliClient, v: &Value, success_message: &str) {
    if c.quiet {
        print_id(v);
    } else if c.is_json() {
        c.print_value(v);
    } else {
        println!("{}", success_message);
        c.print_value(v);
    }
}

pub(crate) fn print_action_result(
    c: &CliClient,
    v: &Value,
    quiet_output: &str,
    success_message: &str,
) {
    if c.quiet {
        println!("{}", quiet_output);
    } else if c.is_json() {
        c.print_value(v);
    } else {
        println!("{}", success_message);
        c.print_value(v);
    }
}

pub(crate) fn print_status_result(c: &CliClient, v: &Value, field: &str, success_message: &str) {
    if c.quiet {
        if let Some(value) = v.get(field).and_then(|value| value.as_str()) {
            println!("{}", value);
        }
    } else if c.is_json() {
        c.print_value(v);
    } else {
        println!("{}", success_message);
        c.print_value(v);
    }
}

pub(crate) fn print_delete_result(c: &CliClient, id: &str, success_message: &str) {
    if c.quiet {
        println!("{}", id);
    } else if c.is_json() {
        c.print_value(&json!({ "id": id, "deleted": true }));
    } else {
        println!("{}", success_message);
    }
}

pub(crate) fn print_download_result(c: &CliClient, id: &str, output: &str) {
    if c.quiet {
        println!("{}", output);
    } else if c.is_json() {
        c.print_value(&json!({ "id": id, "output": output }));
    } else {
        println!("✅ Downloaded to {}", output);
    }
}

pub(crate) fn print_list_response(c: &CliClient, v: &Value) {
    if let Some(count) = v.get("count").and_then(|c| c.as_i64()) {
        if !c.quiet {
            eprintln!("Total: {}", count);
        }
    }
    if let Some(data) = v.get("data") {
        c.print_list_or_item(data);
    } else {
        c.print_value(v);
    }
}

pub(crate) fn print_id(v: &Value) {
    if let Some(id) = v.get("id").and_then(|i| i.as_str()) {
        println!("{}", id);
    } else if let Some(data) = v.get("data") {
        if let Some(id) = data.get("id").and_then(|i| i.as_str()) {
            println!("{}", id);
        }
    }
}
