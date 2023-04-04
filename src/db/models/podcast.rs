use crate::{db_object, db_run};
use crate::db::DbConn;
use utoipa::ToSchema;
use diesel::prelude::{Insertable, Queryable};
use diesel::dsl;

db_object! {
    #[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema)]
    #[diesel(table_name = podcasts)]
    #[diesel(treat_none_as_null = true)]
    #[diesel(primary_key(id))]
pub struct Podcast {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub directory: String,
    #[diesel(sql_type = Text)]
    pub(crate) rssfeed: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Text)]
    pub favored: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub summary: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub language: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub explicit: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub keywords: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_build_date: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    }
}

impl Podcast {
    pub async fn get_podcasts(conn: &mut DbConn) -> Result<Vec<Podcast>, String> {
        db_run! { conn: {
            let result = podcasts::table
            .load::<Self>(conn)
            .expect("Error loading podcasts");
        Ok(result)
            }
    }
    }

    pub async fn find_podcast_by_trackid(conn: &mut DbConn, podcast_id: i32)->Option<Self>{
        db_run!{conn:{
            podcasts::table
            .filter(podcasts::directory.eq(podcast_id.to_string()))
            .first::<Self>(conn)
            .optional()
            .expect("Error loading podcast by id")
        }}
    }

    pub async fn find_podcast(conn: &mut DbConn, podcast_id_to_be_found: i32)->Option<Self>{
        db_run!{conn:{
            let query = podcasts::table
            .filter(podcasts::id.eq(podcast_id_to_be_found));
            query.first::<Self>(conn)
            .optional()
            .expect("Error loading podcast by id")
        }}
    }

    pub async fn add_podcast_to_database(collection_name: String,
                                                 collection_id: String,
                                                 feed_url: String,
                                                 image_url_1: String, conn: &mut DbConn)->Self{
        db_run!{conn:{
            dsl::insert_into(podcasts::table)
            .values((
                podcasts::directory.eq(collection_id.to_string()),
                podcasts::name.eq(collection_name.to_string()),
                podcasts::rssfeed.eq(feed_url.to_string()),
                podcasts::image_url.eq(image_url_1.to_string()),
                podcasts::original_image_url.eq(image_url_1.to_string()),
            ))
            .get_result::<Podcast>(conn)
            .expect("Error inserting podcast")
        }}

    }
}

