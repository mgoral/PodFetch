use chrono::{Duration, NaiveDateTime, Utc};
use utoipa::ToSchema;
use crate::db::DbConn;


db_object! {
    #[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema)]
    #[diesel(table_name = podcast_episodes)]
    #[diesel(treat_none_as_null = true)]
    #[diesel(primary_key(id))]
pub struct PodcastEpisode {
    pub(crate) id: i32,
    pub(crate) podcast_id: i32,
    pub(crate) episode_id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub(crate) local_url: String,
    pub(crate) local_image_url: String,
    pub(crate) description: String,
    pub(crate) status: String,
    pub(crate) download_time: Option<NaiveDateTime>,
}
    }

impl PodcastEpisode {

    pub async fn get_podcast_episodes_older_than_days(&mut conn:DbConn, days:i32) {
        db_run!{
            conn:{
                podcast_episodes::table
                .filter(podcast_episodes::download_time.lt(Utc::now().naive_utc() - Duration::days
                    (days
                    as i64)))
                .load::<PodcastEpisode>(conn)
                .expect("Error loading podcast episode by id")
            }
        }
    }

    pub async fn get_downloaded_episodes(&mut conn:DbConn) {
        db_run!{
            conn:{
                podcast_episodes::table
                .filter(podcast_episodes::status.eq("D"))
                .load::<PodcastEpisode>(conn)
                .expect("Error loading podcast episode by id")
            }
        }
    }

    pub async fn query_for_podcast_episodes(&mut conn:DbConn, query: &str)
        ->Vec<PodcastEpisode> {
        db_run!{
            conn:{
                podcast_episodes::table
                .filter((
                    podcast_episodes::name.like(format!("%{}%", query)
                    .or(podcast_episodes::description.like(format!("%{}%", query))))))
                .load::<PodcastEpisode>(conn)
                .expect("Error loading podcast episode by id")
            }
        }
    }

    pub async fn update_podcast_episode_status(&mut conn:DbConn, download_url_of_episode: &str,
                                               status: &str) {
        db_run!{
            conn:{
                diesel::update(podcast_episodes::table.filter(podcast_episodes::url.eq
                    (download_url_of_episode)))
                .set((
                    podcast_episodes::status.eq(status),
                    podcast_episodes::download_time.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)
                .expect("Error updating podcast episode status");
            }
        }
    }

    pub async fn check_if_downloaded(&mut conn:DbConn, download_url_of_episode: &str) ->bool {
        db_run!{
            conn:{
                let result = podcast_episodes::table
                .filter(podcast_episodes::local_url.is_not_null())
                .filter(podcast_episodes::url.eq(download_url_of_episode))
                .first::<PodcastEpisode>(conn)
                .optional()
                .expect("Error loading podcast episode by id");
                match result {
                    Some(podcast_episode) => {
                        return match podcast_episode.status.as_str() {
                            "N" => false,
                            "D" => true,
                            "P" => false,
                            _ => false,
                        }
                    }
                    None => {
                        panic!("Podcast episode not found");
                    }
                }
            }
        }
    }

    pub async fn get_podcast_episodes_of_podcast(&mut conn:DbConn, podcast_id: i32, last_id: Option<String>) ->Vec<PodcastEpisode> {
        db_run!{
            conn:{
match last_id {
            Some(last_id) => {
                podcast_episodes::table
                    .filter(podcast_episodes::podcast_id.eq(podcast_id))
                    .filter(podcast_episodes::date_of_recording.lt(last_id))
                    .order(podcast_episodes::date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts")
            }
            None => {
                podcast_episodes::table.filter(podcast_episodes::podcast_id.eq(podcast_id))
                    .order(podcast_episodes::date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts")
            }
        }
            }
        }
    }

    pub async fn get_last_5_podcast_episodes_of_podcast(&mut conn:DbConn, podcast_id: i32) ->Vec<PodcastEpisode> {
        db_run!{
            conn:{
                podcast_episodes::table
                    .filter(podcast_episodes::podcast_id.eq(podcast_id))
                    .order(podcast_episodes::date_of_recording.desc())
                    .limit(5)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts")
            }
        }
    }

    pub async fn add_podcast_episode(&mut conn:DbConn, podcast_episode: PodcastEpisode) {
        db_run!{
            conn:{
                diesel::insert_into(podcast_episodes::table)
                .values(&podcast_episode)
                .execute(conn)
                .expect("Error saving new podcast episode");
            }
        }
    }

}