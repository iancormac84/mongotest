use mongodb::{db::ThreadedDatabase, oid::ObjectId, ThreadedClient};
use mongotest::{AppState, FooInput, Result, ThingInput};
use serde_json::json;

#[runtime::main]
async fn main() -> Result<()> {
    let client = surf::Client::new();
    let mutation = r#"mutation AddFoo($input: [FooInput!]!) {
        addFoo(input: $input)
    }"#;
    let value_json = json!({
        "query": mutation,
        "variables": {
            "input": FooInput {
                foo_string: "Random".to_string(),
                things: vec![
                ThingInput {
                    id: ObjectId::new().unwrap(),
                    thing_info: "Random1".to_string(),
                },
                ThingInput {
                    id: ObjectId::new().unwrap(),
                    thing_info: "Random2".to_string(),
                }],
            }
        }
    });
    println!("{}", value_json);
    let mut res = client
        .post("http://127.0.0.1:8000/graphql")
        .body_json(&value_json)?
        .await?;
    println!("res is {:?}", res);
    let content: serde_json::Value = res.body_json().await?;
    println!("content is {}", content);
    let input = FooInput {
        foo_string: "Random".to_string(),
        things: vec![
            ThingInput {
                id: ObjectId::new().unwrap(),
                thing_info: "Random1".to_string(),
            },
            ThingInput {
                id: ObjectId::new().unwrap(),
                thing_info: "Random2".to_string(),
            },
        ],
    };
    let mongo_client =
        mongodb::Client::connect("localhost", 27017).expect("Failed to initialize client.");
    let app_state = AppState::new(mongo_client, "foos_and_things");
    let foos = app_state.database.collection("foos");
    let input = match mongodb::to_bson(&input) {
        Ok(o) => o,
        Err(err) => return Err(err.into()),
    };
    let input = input.as_document().cloned();
    let input = input?;
    let res = foos.insert_one(input, None);
    match res {
        Ok(r) => {
            if let Some(bs) = r.inserted_id {
                let oid = bs.as_object_id()?;
                println!("oid is {}", oid);
            } else if let Some(we) = r.write_exception {
                return Err(we.into());
            } else {
                unreachable!()
            }
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}
