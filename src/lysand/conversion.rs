use activitypub_federation::fetch::object_id::ObjectId;
use activitystreams_kinds::public;
use sea_orm::{EntityTrait, QueryFilter};
use url::Url;

use crate::{database::State, entities::{self, post, prelude}, objects::post::Mention, utils::{generate_object_id, generate_user_id}, FEDERATION_CONFIG};

use super::objects::Note;

pub async fn receive_lysand_note(note: Note, db_id: String, db: State) {
    let author: entities::user::Model = todo!();
    let user_res = prelude::User::find_by_id(db_id).one(db.database_connection.as_ref()).await;
    if user_res.is_err() {
        println!("{}", user_res.unwrap_err());
        return;
    }
    if let Some(target) = user_res.ok().unwrap() {
        let data = FEDERATION_CONFIG.get().unwrap();
        let id: ObjectId<post::Model> = generate_object_id(data.domain(), &note.id.to_string()).unwrap().into();
        let user_id = generate_user_id(data.domain(), &target.id.to_string()).unwrap();
        let to = match note.visibility.unwrap_or(super::objects::VisibilityType::Public) {
            super::objects::VisibilityType::Public => vec![public(), Url::parse(&author.inbox).unwrap()],
            super::objects::VisibilityType::Followers => vec![Url::parse(&author.inbox).unwrap()],
            super::objects::VisibilityType::Direct => vec![user_id],
            super::objects::VisibilityType::Unlisted => vec![Url::parse(&author.inbox).unwrap()],
        };
        let cc = match note.visibility.unwrap_or(super::objects::VisibilityType::Public) {
            super::objects::VisibilityType::Unlisted => Some(vec![public()]),
            _ => None
        };
        let mut tag: Vec<Mention> = Vec::new();
        for l_tag in note.mentions.unwrap_or_default() {
            tag.push(Mention { href: l_tag, //todo convert to ap url
                kind: Default::default(), })
        }
        let ap_note = crate::objects::post::Note {
            kind: Default::default(),
            id,
            sensitive: note.is_sensitive.unwrap_or(false),
            cc,
            to,
            tag,
            
        }
    }
}