use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject};
use aws_sdk_dynamodb::{types::AttributeValue, Client};

const TABLE: &str = "Music";
const ARTIST: &str = "Artist";
const SONGTITLE: &str = "SongTitle";

fn as_s(v: Option<&AttributeValue>) -> String {
    if let Some(v) = v {
        if let Ok(s) = v.as_s() {
            return s.to_owned();
        }
    }
    return String::from("");
}

impl From<&HashMap<String, AttributeValue>> for Song {
    fn from(v: &HashMap<String, AttributeValue>) -> Self {
        Song {
            title: as_s(v.get(SONGTITLE)),
            artist: as_s(v.get(ARTIST)),
        }
    }
}

#[derive(SimpleObject)]
pub struct Song {
    pub title: String,
    pub artist: String,
}

pub(crate) struct Query;
pub(crate) struct Mutation;

#[Object]
impl Query {
    async fn songs<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Song>, async_graphql::Error> {
        let db = ctx.data::<Client>();
        if let Ok(db) = db {
            let res = db.scan().table_name(TABLE).limit(10).send().await?;
            if let Some(items) = res.items {
                let songs: Vec<Song> = items.iter().map(|v| v.into()).collect();
                return Ok(songs);
            }
        }
        return Ok(vec![]);
    }
}

#[Object]
impl Mutation {
    async fn create_song<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        title: String,
        artist: String,
    ) -> Result<Song, async_graphql::Error> {
        let db = ctx.data::<Client>();
        if let Ok(db) = db {
            db.put_item()
                .table_name(TABLE)
                .item(SONGTITLE, AttributeValue::S(title.clone()))
                .item(ARTIST, AttributeValue::S(artist.clone()))
                .send()
                .await?;
            return Ok(Song { title, artist });
        }
        return Err(async_graphql::Error {
            message: String::from("no db"),
            source: Option::None,
            extensions: Option::None,
        });
    }
}
