use crate::{
    activities::{create_post::CreatePost, follow::Follow},
    entities::{
        self, follow_relation,
        prelude::{self, FollowRelation},
        user,
    },
    utils::generate_follow_req_id,
    versia::http::main_versia_url_to_user_and_model,
    API_DOMAIN, DB, FEDERATION_CONFIG,
};
use activitypub_federation::{
    activity_sending::SendActivityTask, fetch::object_id::ObjectId, protocol::context::WithContext,
};
use activitystreams_kinds::{activity::FollowType, public};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityOrSelect, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use url::Url;

use super::{
    conversion::{db_user_from_url, fetch_user_from_url, receive_versia_note, versia_user_from_db},
    http::{versia_url_to_user, versia_url_to_user_and_model},
};

pub async fn inbox_entry(json: &str) -> Result<()> {
    // Deserialize the JSON string into a dynamic value
    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    // Extract the "type" field from the JSON
    if let Some(json_type) = value.get("type") {
        // Match the "type" field with the corresponding VersiaType
        match json_type.as_str() {
            Some("Note") => {
                let note: super::objects::Note = serde_json::from_str(json)?;
                federate_inbox(note).await?;
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
            Some("Unfollow") => {
                let unfollow: super::objects::Unfollow = serde_json::from_str(json)?;
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
    let author = main_versia_url_to_user_and_model(follow.author.into()).await?;
    println!("Followee URL: {}", &follow.followee.to_string());
    let followee = versia_url_to_user_and_model(follow.followee.into()).await?;
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

async fn federate_inbox(note: super::objects::Note) -> Result<()> {
    let db_user = db_user_from_url(note.author.clone()).await?;
    let note = receive_versia_note(note, db_user.id).await?;

    let ap_str = note.ap_json.clone().unwrap();
    let ap_note = serde_json::from_str::<crate::objects::post::Note>(&ap_str)?;

    tokio::spawn(async move {
        let conf = FEDERATION_CONFIG.get().unwrap();
        let inbox = get_inbox_vec(&ap_note).await;

        let res = CreatePost::sends(ap_note, note, inbox, &conf.to_request_data()).await;
        if let Err(e) = res {
            panic!("Problem federating: {e:?}");
        }
    });

    Ok(())
}

async fn get_inbox_vec(ap_note: &crate::objects::post::Note) -> Vec<Url> {
    let mut inbox_users: Vec<Url> = Vec::new();
    let mut inbox: Vec<Url> = Vec::new();

    let entry = ap_note.to.get(0).unwrap();
    if entry
        .to_string()
        .eq_ignore_ascii_case(public().to_string().as_str())
    {
        let (_, mentions) = ap_note.to.split_at(2);
        inbox_users.append(&mut mentions.to_vec());
    } else {
        let (_, mentions) = ap_note.to.split_at(1);
        inbox_users.append(&mut mentions.to_vec());
    }

    inbox_users.dedup();

    let conf = FEDERATION_CONFIG.get().unwrap();
    let data = &conf.to_request_data();

    for user in inbox_users {
        let ap_user = ObjectId::<user::Model>::from(user)
            .dereference(data)
            .await
            .unwrap();
        inbox.push(Url::parse(&ap_user.inbox).unwrap());
    }

    inbox.dedup();

    inbox
}
