extern crate serde; // 1.0.68
extern crate serde_derive; // 1.0.68

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use time::{
    format_description::well_known::{iso8601, Iso8601},
    OffsetDateTime,
};
use url::Url;
use uuid::Uuid;

const FORMAT: Iso8601<6651332276412969266533270467398074368> = Iso8601::<
    {
        iso8601::Config::DEFAULT
            .set_year_is_six_digits(false)
            .encode()
    },
>;
time::serde::format_description!(iso_lysand, OffsetDateTime, FORMAT);

fn sort_alphabetically<T: Serialize, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}

#[derive(Serialize)]
pub struct SortAlphabetically<T: Serialize>(#[serde(serialize_with = "sort_alphabetically")] pub T);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LysandType {
    User,
    Note,
    Patch,
    Like,
    Dislike,
    Follow,
    FollowAccept,
    FollowReject,
    Undo,
    Extension,
    ServerMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CategoryType {
    Microblog,
    Forum,
    Blog,
    Image,
    Video,
    Audio,
    Messaging
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VisibilityType {
    Public,
    Unlisted,
    Followers,
    Direct
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LysandExtensions {
    #[serde(rename = "org.lysand:microblogging/Announce")]
    Announce,
    #[serde(rename = "org.lysand:custom_emojis")]
    CustomEmojis,
    #[serde(rename = "org.lysand:reactions/Reaction")]
    Reaction,
    #[serde(rename = "org.lysand:reactions")]
    Reactions,
    #[serde(rename = "org.lysand:polls")]
    Polls,
    #[serde(rename = "org.lysand:is_cat")]
    IsCat,
    #[serde(rename = "org.lysand:server_endorsement/Endorsement")]
    Endorsement,
    #[serde(rename = "org.lysand:server_endorsement")]
    EndorsementCollection,
    #[serde(rename = "org.lysand:reports/Report")]
    Report,
    #[serde(rename = "org.lysand:vanity")]
    Vanity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicKey {
    public_key: String,
    actor: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentHash {
    md5: Option<String>,
    sha1: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
}

#[derive(Debug, Clone)]
struct ContentFormat {
    x: HashMap<String, ContentEntry>,
}

impl Serialize for ContentFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(Some(self.x.len()))?;
        for (k, v) in &self.x {
            seq.serialize_entry(&k.to_string(), &v)?;
        }
        seq.end()
    }
}
impl<'de> Deserialize<'de> for ContentFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::deserialize(deserializer)?;
        Ok(ContentFormat { x: map })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FieldKV {
    name: ContentFormat,
    value: ContentFormat,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentEntry {
    content: String,
    description: Option<String>,
    size: Option<u64>,
    hash: Option<ContentHash>,
    blurhash: Option<String>,
    fps: Option<u64>,
    width: Option<u64>,
    height: Option<u64>,
    duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub public_key: PublicKey,
    #[serde(rename = "type")]
    pub rtype: LysandType,
    pub id: Uuid,
    pub uri: Url,
    #[serde(with = "iso_lysand")]
    pub created_at: OffsetDateTime,
    pub display_name: Option<String>,
    // TODO bio: Option<String>,
    pub inbox: Url,
    pub outbox: Url,
    pub featured: Url,
    pub followers: Url,
    pub following: Url,
    pub likes: Url,
    pub dislikes: Url,
    pub username: String,
    pub bio: Option<ContentFormat>,
    pub avatar: Option<ContentFormat>,
    pub header: Option<ContentFormat>,
    pub fields: Option<Vec<FieldKV>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    name: String,
    version: String,
    url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkPreview {
    description: String,
    title: String,
    link: Url,
    image: Option<Url>,
    icon: Option<Url>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    #[serde(rename = "type")]
    pub rtype: LysandType,
    pub id: Uuid,
    pub uri: Url,
    pub author: Url,
    #[serde(with = "iso_lysand")]
    pub created_at: OffsetDateTime,
    pub category: Option<CategoryType>,
    pub content: Option<ContentFormat>,
    pub device: Option<DeviceInfo>,
    pub previews: Option<Vec<LinkPreview>>,
    pub group: Option<String>,
    pub attachments: Option<Vec<ContentFormat>>,
    pub replies_to: Option<Url>,
    pub quotes: Option<Url>,
    pub mentions: Option<Vec<Url>>,
    pub subject: Option<String>,
    pub is_sensitive: Option<bool>,
    pub visibility: Option<VisibilityType>,
    //TODO extensions
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Outbox {
    pub first: Url,
    pub last: Url,
    pub next: Option<Url>,
    pub prev: Option<Url>,
    pub items: Vec<Note>,
}