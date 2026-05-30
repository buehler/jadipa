use jadipa::diff;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = json!({
        "title": "Draft release notes",
        "status": "draft",
        "tags": ["release", "internal"],
        "metadata": {
            "owner": "core",
            "reviewed": false
        },
        "temporary": true
    });
    let target = json!({
        "title": "Draft release notes",
        "status": "published",
        "tags": ["release", "json-patch", "internal"],
        "metadata": {
            "owner": "core",
            "reviewed": true
        },
        "publishedAt": "2026-05-30"
    });

    let patch = diff::diff(&source, &target);
    let patched = patch.apply(&source)?;

    assert_eq!(patched, target);
    assert_eq!(
        source,
        json!({
            "title": "Draft release notes",
            "status": "draft",
            "tags": ["release", "internal"],
            "metadata": {
                "owner": "core",
                "reviewed": false
            },
            "temporary": true
        })
    );

    println!("Source document:");
    println!("{}", serde_json::to_string_pretty(&source)?);
    println!();
    println!("Target document:");
    println!("{}", serde_json::to_string_pretty(&target)?);
    println!();
    println!("Generated diff patch:");
    println!("{}", serde_json::to_string_pretty(&patch)?);
    println!();
    println!("Patched document:");
    println!("{}", serde_json::to_string_pretty(&patched)?);

    Ok(())
}
