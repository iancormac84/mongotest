#![feature(try_trait)]

use http::{header::HeaderValue, status::StatusCode};
use juniper::{graphql_object, graphql_value, FieldError, GraphQLInputObject, GraphQLObject};
use mongodb::{
    db::{Database, ThreadedDatabase},
    doc,
    oid::ObjectId,
    Client, ThreadedClient,
};
use serde::{Deserialize, Serialize};
use tide::{error::ResultExt, response, Context, EndpointResult, Response};

mod error;
mod query;
pub use crate::error::Result;
pub use crate::query::Query;

// Adding `Query` and `Mutation` together we get `Schema`, which describes, well, the whole GraphQL
// schema.
type Schema = juniper::RootNode<'static, Query, Mutation>;

// Finally, we'll bridge between Tide and Juniper. `GraphQLRequest` from Juniper implements
// `Deserialize`, so we use `Json` extractor to deserialize the request body.
pub async fn handle_graphql(mut cx: Context<AppState>) -> EndpointResult {
    let query: juniper::http::GraphQLRequest = cx.body_json().await.client_err()?;
    let schema = Schema::new(Query, Mutation);
    let response = query.execute(&schema, cx.state());
    let status = if response.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let mut resp = response::json(response);
    *resp.status_mut() = status;
    Ok(resp)
}

pub async fn graphiql(mut _cx: Context<AppState>) -> EndpointResult {
    let html = juniper::http::graphiql::graphiql_source("/graphql");
    let mut resp = Response::new(html.into());
    let headers = resp.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(resp)
}

pub struct AppState {
    pub client: Client,
    pub db_name: String,
    pub database: Database,
}

impl AppState {
    pub fn new<S>(client: Client, db_name: S) -> AppState
    where
        S: Into<String>,
    {
        let db_name = db_name.into();
        let database = client.db(&db_name);
        let app_state = AppState {
            client,
            db_name,
            database,
        };
        app_state
    }
    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

impl juniper::Context for AppState {}

#[derive(Debug, Serialize, Deserialize, GraphQLInputObject)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct FooInput {
    pub foo_string: String,
    pub things: Vec<ThingInput>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct FooPayload {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub foo_string: String,
    pub things: Vec<ThingInput>,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, GraphQLInputObject)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ThingInput {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub thing_info: String,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct UpdateStats {
    pub id: Option<ObjectId>,
    pub matched_count: i32,
    pub modified_count: i32,
}

pub struct Mutation;

graphql_object!(Mutation: AppState |&self| {
    field add_foo(&executor, input: FooInput) -> juniper::FieldResult<ObjectId> as "Insert a new Foo document." {
        let app_state = &executor.context();
        let foos = app_state.database.collection("foos");

        let input = match mongodb::to_bson(&input) {
            Ok(o) => o,
            Err(err) => return Err(FieldError::new(err, juniper::Value::null())),
        };
        let input = input.as_document().cloned();
        let input = if let Some(o) = input {
            o
        } else {
            return Err(FieldError::new("Could not find a valid Foo document.", graphql_value!({ "document_error": "Document creation failed" })));
        };
        let res = foos.insert_one(input, None);
        match res {
            Ok(r) => {
                if let Some(bs) = r.inserted_id {
                    match bs.as_object_id() {
                        Some(id) => return Ok(id.clone()),
                        None => return Err(FieldError::new("Invalid ObjectId", juniper::Value::null()))
                    }
                } else if let Some(we) = r.write_exception {
                    return Err(FieldError::new(we, juniper::Value::null()));
                } else {
                    unreachable!()
                }
            }
            Err(err) => Err(FieldError::new(err, juniper::Value::null()))
        }
    }
    field add_foos(&executor, input: Vec<FooInput>) -> juniper::FieldResult<Vec<ObjectId>> as "Insert a list of Foos into the database." {
        let app_state = &executor.context();
        let foos = app_state.database.collection("foos");

        let bson_input = input.into_iter().map(|s| {
            let sjb = mongodb::to_bson(&s).unwrap();
            let sjb = sjb.as_document().cloned().unwrap();
            sjb
        }).collect();
        
        let res = foos.insert_many(bson_input, None);
        match res {
            Ok(r) => {
                if let Some(bs) = r.inserted_ids {
                    let ids = bs.values().into_iter().map(|b| b.as_object_id().unwrap().clone()).collect();
                    Ok(ids)
                } else if let Some(we) = r.bulk_write_exception {
                    return Err(FieldError::new(we, juniper::Value::null()));
                } else {
                    unreachable!()
                }
            }
            Err(err) => Err(FieldError::new(err, juniper::Value::null()))
        }
    }
    field delete_foos(&executor, foo_id: ObjectId) -> juniper::FieldResult<bool> {
        let app_state = &executor.context();
        let foos = app_state.database.collection("foos");

        let doc = foos.find_one(Some(doc!{"_id": foo_id}), None)?;
        if let Some(doc) = doc {
            let res = foos.delete_one(doc, None);

            match res {
                Ok(r) => {
                    if r.deleted_count > 0 {
                        return Ok(true);
                    } else {
                        if let Some(we) = r.write_exception {
                            return Err(FieldError::new(we, juniper::Value::null()));
                        } else {
                            return Ok(false);
                        }
                    }
                }
                Err(err) => Err(FieldError::new(err, juniper::Value::null()))
            }
        } else {
            Ok(false)
        }
    }
    field add_thing(&executor, foo_id: ObjectId, input: ThingInput) -> juniper::FieldResult<UpdateStats> as "Insert a new Thing document." {
        let app_state = &executor.context();
        let foos = app_state.database.collection("foos");

        //Taking the chance of just calling unwrap on the next two lines because Rust should have checked that they are valid
        let input = mongodb::to_bson(&input).unwrap();
        let input = input.as_document().cloned().unwrap();
        let res = foos.update_one(doc!{"_id": foo_id}, doc!{"$push": { "things": input}}, None);
        match res {
            Ok(r) => {
                if let Some(we) = r.write_exception {
                    return Err(FieldError::new(we, juniper::Value::null()));
                } else {
                    Ok(UpdateStats {
                        id: if let Some(bs) = r.upserted_id {
                            Some(bs.as_object_id().unwrap().clone())
                        } else {
                            None
                        },
                        matched_count: r.matched_count,
                        modified_count: r.modified_count,
                    })
                }
            }
            Err(err) => Err(FieldError::new(err, juniper::Value::null()))
        }
    }
});
