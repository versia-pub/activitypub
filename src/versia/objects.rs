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
time::serde::format_description!(iso_versia, OffsetDateTime, FORMAT);

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
pub enum CategoryType {
    Microblog,
    Forum,
    Blog,
    Image,
    Video,
    Audio,
    Messaging,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VersiaExtensions {
    #[serde(rename = "pub.versia:share/Share")]
    Share,
    #[serde(rename = "pub.versia:custom_emojis")]
    CustomEmojis,
    #[serde(rename = "pub.versia:reactions/Reaction")]
    Reaction,
    #[serde(rename = "pub.versia:reactions")]
    Reactions,
    #[serde(rename = "pub.versia:polls")]
    Polls,
    #[serde(rename = "pub.versia:is_cat")]
    IsCat,
    #[serde(rename = "pub.versia:server_endorsement/Endorsement")]
    Endorsement,
    #[serde(rename = "pub.versia:server_endorsement")]
    EndorsementCollection,
    #[serde(rename = "pub.versia:reports/Report")]
    Report,
    #[serde(rename = "pub.versia:vanity")]
    Vanity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicKey {
    pub key: String,
    pub actor: Url,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentHash {
    md5: Option<String>,
    sha1: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ContentFormat {
    pub x: HashMap<String, ContentEntry>,
}

impl ContentFormat {
    pub async fn select_rich_text(&self) -> anyhow::Result<String> {
        if let Some(entry) = self.x.get("text/x.misskeymarkdown") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("text/html") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("text/markdown") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("text/plain") {
            return Ok(entry.content.clone());
        }

        Ok(self.x.clone().values().next().unwrap().content.clone())
    }

    pub async fn select_rich_img(&self) -> anyhow::Result<String> {
        if let Some(entry) = self.x.get("image/webp") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/png") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/avif") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/jxl") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/jpeg") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/gif") {
            return Ok(entry.content.clone());
        }
        if let Some(entry) = self.x.get("image/bmp") {
            return Ok(entry.content.clone());
        }

        Ok(self.x.clone().values().next().unwrap().content.clone())
    }

    pub async fn select_rich_img_touple(&self) -> anyhow::Result<(String, String)> {
        if let Some(entry) = self.x.get("image/webp") {
            return Ok(("image/webp".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/png") {
            return Ok(("image/png".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/avif") {
            return Ok(("image/avif".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/jxl") {
            return Ok(("image/jxl".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/jpeg") {
            return Ok(("image/jpeg".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/gif") {
            return Ok(("image/gif".to_string(), entry.content.clone()));
        }
        if let Some(entry) = self.x.get("image/bmp") {
            return Ok(("image/bmp".to_string(), entry.content.clone()));
        }

        let touple = self.x.iter().next().unwrap();

        Ok((touple.0.clone(), touple.1.content.clone()))
    }
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
pub struct FieldKV {
    pub key: ContentFormat,
    pub value: ContentFormat,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentEntry {
    content: String,
    remote: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash: Option<ContentHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blurhash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<u64>,
}
impl ContentEntry {
    pub fn from_string(string: String) -> ContentEntry {
        ContentEntry {
            content: string,
            remote: false,
            description: None,
            size: None,
            hash: None,
            blurhash: None,
            fps: None,
            width: None,
            height: None,
            duration: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub public_key: PublicKey,
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub uri: Url,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub collections: UserCollections,
    pub inbox: Url,
    pub likes: Url,
    pub dislikes: Url,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<ContentFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<ContentFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<ContentFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldKV>>,
    pub indexable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionSpecs>,
    pub manually_approves_followers: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCollections {
    pub outbox: Url,
    pub featured: Url,
    pub followers: Url,
    pub following: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionSpecs {
    #[serde(rename = "pub.versia:custom_emojis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_emojis: Option<CustomEmojis>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomEmojis {
    pub emojis: Vec<CustomEmoji>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomEmoji {
    pub name: String,
    pub url: ContentFormat,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Url>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub uri: Url,
    pub author: Url,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CategoryType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ContentFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DeviceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previews: Option<Vec<LinkPreview>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<ContentFormat>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies_to: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<Url>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sensitive: Option<bool>,
    //TODO extensions
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Outbox {
    pub first: Url,
    pub last: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<Url>,
    pub items: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Follow {
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub uri: Url,
    pub author: Url,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    pub followee: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FollowResult {
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub uri: Url,
    pub author: Url,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    pub follower: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Unfollow {
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub author: Url,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    pub followee: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Delete {
    #[serde(rename = "type")]
    pub rtype: String,
    pub id: Uuid,
    pub author: Option<Url>,
    #[serde(with = "iso_versia")]
    pub created_at: OffsetDateTime,
    pub deleted_type: String,
    pub deleted: Url,
}
