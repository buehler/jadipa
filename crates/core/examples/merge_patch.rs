use jadipa::merge_patch;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = json!({
        "title": "Goodbye!",
        "author": {
            "givenName": "John",
            "familyName": "Doe"
        },
        "tags": ["example", "sample"],
        "content": "This will be unchanged"
    });
    let patch = json!({
        "title": "Hello!",
        "phoneNumber": "+01-123-456-7890",
        "author": {
            "familyName": null
        },
        "tags": ["example"]
    });

    let patched = merge_patch::apply(&target, &patch);
    let expected = json!({
        "title": "Hello!",
        "author": {
            "givenName": "John"
        },
        "tags": ["example"],
        "content": "This will be unchanged",
        "phoneNumber": "+01-123-456-7890"
    });

    assert_eq!(patched, expected);

    println!("Original document:");
    println!("{}", serde_json::to_string_pretty(&target)?);
    println!();
    println!("Merge patch:");
    println!("{}", serde_json::to_string_pretty(&patch)?);
    println!();
    println!("Patched document:");
    println!("{}", serde_json::to_string_pretty(&patched)?);

    Ok(())
}
