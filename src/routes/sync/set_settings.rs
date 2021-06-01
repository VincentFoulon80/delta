use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use chrono::prelude::*;
use rocket::request::Form;
use std::collections::HashMap;
use rocket_contrib::json::Json;
use mongodb::bson::{doc, to_bson};
use serde::{Serialize, Deserialize};
use mongodb::options::UpdateOptions;

type Data = HashMap<String, String>;

#[derive(Serialize, Deserialize, FromForm)]
pub struct Options {
    timestamp: Option<i64>,
}

#[post("/settings/set?<options..>", data = "<data>")]
pub async fn req(user: User, data: Json<Data>, options: Form<Options>) -> Result<()> {
    let data = data.into_inner();
    let current_time = Utc::now().timestamp_millis();
    let timestamp = if let Some(timestamp) = options.timestamp {
        if timestamp > current_time {
            current_time
        } else {
            timestamp
        }
    } else {
        current_time
    };

    let mut set = doc! {};
    for (key, data) in &data {
        set.insert(
            key.clone(),
            vec! [
                to_bson(&timestamp).unwrap(),
                to_bson(&data.clone()).unwrap()
            ]
        );
    }

    if set.len() > 0 {
        get_collection("user_settings")
            .update_one(
                doc! {
                    "_id": &user.id
                },
                doc! {
                    "$set": &set
                },
                UpdateOptions::builder()
                    .upsert(true)
                    .build()
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "user_settings" })?;
    }

    ClientboundNotification::UserSettingsUpdate {
        id: user.id.clone(),
        update: json!(set)
    }
    .publish(user.id);
    
    Ok(())
}