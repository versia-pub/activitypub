use crate::{
    activities::follow::Follow,
    entities::{
        self, follow_relation,
        prelude::{self, FollowRelation},
        user,
    },
    lysand::http::main_lysand_url_to_user_and_model,
    utils::generate_follow_req_id,
    API_DOMAIN, DB, FEDERATION_CONFIG,
};
use activitypub_federation::{
    activity_sending::SendActivityTask, fetch::object_id::ObjectId, protocol::context::WithContext,
};
use activitystreams_kinds::activity::FollowType;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityOrSelect, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use url::Url;

use super::{
    conversion::lysand_user_from_db,
    http::{lysand_url_to_user, lysand_url_to_user_and_model},
    objects::LysandType,
};

pub async fn inbox_entry(json: &str) -> Result<()> {
    // Deserialize the JSON string into a dynamic value
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    // Extract the "type" field from the JSON
    if let Some(json_type) = value.get("type") {
        // Match the "type" field with the corresponding LysandType
        match json_type.as_str() {
            Some("Note") => {
                let note: super::objects::Note = serde_json::from_str(json)?;
            }
            Some("Patch") => {
                let patch: super::objects::Patch = serde_json::from_str(json)?;
            }
            Some("Follow") => {
                let follow_req: super::objects::Follow = serde_json::from_str(json)?;
                follow_request(follow_req).await?;
            }
            Some("FollowAccept") => {
                let follow_accept: super::objects::FollowResult = serde_json::from_str(json)?;
            }
            Some("FollowReject") => {
                let follow_rej: super::objects::FollowResult = serde_json::from_str(json)?;
            }
            // Add more cases for other types as needed
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown 'type' field in JSON, it is {}",
                    json_type
                ));
            }
        }
    } else {
        return Err(anyhow::anyhow!("Missing 'type' field in JSON"));
    }
    Ok(())
}

async fn follow_request(follow: super::objects::Follow) -> Result<()> {
    // Check if the user is already following the requester
    let db = DB.get().unwrap();
    let query = FollowRelation::find()
        .filter(follow_relation::Column::FollowerId.eq(follow.author.to_string().as_str()))
        .filter(follow_relation::Column::FolloweeId.eq(follow.followee.to_string().as_str()))
        .one(db)
        .await?;
    if query.is_some() {
        return Err(anyhow::anyhow!(
            "User is already follow requesting / following the followee"
        ));
    }
    let data = FEDERATION_CONFIG.get().unwrap();
    let author = main_lysand_url_to_user_and_model(follow.author.into()).await?;
    println!("Followee URL: {}", &follow.followee.to_string());
    let followee = lysand_url_to_user_and_model(follow.followee.into()).await?;
    let serial_ap_author = serde_json::from_str::<crate::objects::person::Person>(
        &(author.1.ap_json.clone()).unwrap(),
    )?;
    let serial_ap_followee = serde_json::from_str::<crate::objects::person::Person>(
        &(followee.1.ap_json.clone()).unwrap(),
    )?;

    let id = uuid::Uuid::now_v7().to_string();

    let followee_object: ObjectId<user::Model> = serial_ap_followee.id;
    let localuser_object: ObjectId<user::Model> = serial_ap_author.id;

    println!(
        "Sending follow request to {}",
        &followee.0.display_name.unwrap_or(followee.0.username)
    );
    let create = Follow {
        actor: localuser_object.clone(),
        object: followee_object.clone(),
        kind: FollowType::Follow,
        id: generate_follow_req_id(&API_DOMAIN.to_string(), id.clone().as_str())?,
    };

    let ap_json = serde_json::to_string(&create)?;

    let create_with_context = WithContext::new_default(create);

    let follow_db_entry = follow_relation::ActiveModel {
        id: Set(id.clone()),
        followee_id: Set(followee.0.id.to_string()),
        follower_id: Set(author.0.id.to_string()),
        ap_id: Set(Some(id.clone())),
        ap_json: Set(ap_json),
        remote: Set(false),
        ..Default::default()
    };
    follow_db_entry.insert(db).await?;

    let sends = SendActivityTask::prepare(
        &create_with_context,
        &author.1,
        vec![serial_ap_followee.inbox],
        &data.to_request_data(),
    )
    .await?;

    for send in sends {
        send.sign_and_send(&data.to_request_data()).await?;
    }

    Ok(())
}
