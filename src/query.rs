use crate::{AppState, FooPayload};
use juniper::graphql_object;
use mongodb::{bson, db::ThreadedDatabase};

pub struct Query;

graphql_object!(Query: AppState |&self| {
    field foo_payloads(&executor) -> juniper::FieldResult<Vec<FooPayload>> {
        let app_state = &executor.context();
        let foos = app_state.database.collection("foos");
        let cursor = foos.find(None, None)?;
        let res: Result<Vec<FooPayload>, mongodb::Error> = cursor
            .map(|row| row.and_then(|item| Ok(bson::from_bson::<FooPayload>(bson::Bson::Document(item))?)))
            .collect();
        Ok(res?)
    }
});
