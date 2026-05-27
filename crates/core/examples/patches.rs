use jadipa::patch::Patch;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = json!({
        "title": "Draft release notes",
        "status": "draft",
        "tags": ["release", "internal"],
        "metadata": {
            "owner": "core",
            "reviewed": false
        },
        "temporary": true
    });

    let patch = Patch::new(
        r#"[
            {"op":"test","path":"/status","value":"draft"},
            {"op":"replace","path":"/status","value":"published"},
            {"op":"add","path":"/tags/-","value":"json-patch"},
            {"op":"remove","path":"/temporary"},
            {"op":"copy","from":"/metadata/owner","path":"/owner"},
            {"op":"move","from":"/title","path":"/headline"}
        ]"#,
    )?;

    let patched = patch.apply(&target)?;

    let expected = json!({
        "headline": "Draft release notes",
        "status": "published",
        "tags": ["release", "internal", "json-patch"],
        "metadata": {
            "owner": "core",
            "reviewed": false
        },
        "owner": "core"
    });

    assert_eq!(patched, expected);
    assert_eq!(
        target,
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

    println!("Original document:");
    println!("{}", serde_json::to_string_pretty(&target)?);
    println!();
    println!("Patched document:");
    println!("{}", serde_json::to_string_pretty(&patched)?);

    Ok(())
}
