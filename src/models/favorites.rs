use diesel::insert_into;
use diesel::prelude::*;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::dbconfig::schema::favorites;
use serde::{Serialize, Deserialize};
use diesel::sql_types::{Text, Integer, Bool};
use crate::DbConnection;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::service::mapping_service::MappingService;

#[derive(Queryable, Associations, Debug, PartialEq,QueryableByName, Serialize, Deserialize, Insertable,
Clone,
AsChangeset)]
#[diesel(belongs_to(Podcast, foreign_key = podcast_id))]
#[diesel(belongs_to(User, foreign_key = username))]
pub struct Favorite{
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub favored: bool
}

impl Favorite{
    pub fn delete_by_username(username1: String, conn: &mut DbConnection) -> Result<(),
        diesel::result::Error>{
        use crate::dbconfig::schema::favorites::dsl::*;
        diesel::delete(favorites.filter(username.eq(username1))).execute(conn)?;
        Ok(())
    }

    pub fn update_podcast_favor(podcast_id_1: &i32, favor: bool,
                                conn: &mut DbConnection, username_1: String) ->
                                Result<(), String> {
        use crate::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::dbconfig::schema::favorites::dsl::podcast_id;
        use crate::dbconfig::schema::favorites::dsl::username;

        let res = f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1.clone())))
            .first::<Favorite>(conn)
            .optional().unwrap();

        match res{
            Some(..) => {
                diesel::update(f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1))))
                    .set(favor_column.eq(favor))
                    .execute(conn).expect("Error updating podcast");
                Ok(())
            }
            None => {
                insert_into(f_db)
                    .values((
                        podcast_id.eq(podcast_id_1),
                        username.eq(username_1),
                        favor_column.eq(favor),
                    ))
                    .execute(conn).map_err(|e| e.to_string()).unwrap();
                Ok(())
            }
        }
    }

    pub fn get_favored_podcasts(found_username: String,conn:&mut DbConnection,
                                mapping_service:MappingService) ->
    Result<Vec<PodcastDto>,
        String> {
        use crate::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;
        use crate::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::dbconfig::schema::favorites::dsl::username as user_favor;
        use crate::dbconfig::schema::favorites::dsl::favored as favor_column;
        let result:Vec<(Podcast, Favorite)>;

        result = dsl_podcast
            .inner_join(f_db)
            .filter(
                favor_column.eq(true).and(
                    user_favor.eq(found_username)))
            .load::<(Podcast, Favorite)>(conn).unwrap();



        let mapped_result = result
            .iter()
            .map(|podcast| return mapping_service.map_podcast_to_podcast_dto_with_favorites_option
            (&*podcast))
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn search_podcasts_favored(conn: &mut DbConnection, order: OrderCriteria, title: Option<String>,
                                   latest_pub: OrderOption,
                                   designated_username: String) ->Vec<(Podcast, Favorite)>{
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::id as podcastsid;


        let mut query = podcasts.inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .inner_join(favorites::table.on(podcastsid.eq(favorites::dsl::podcast_id).and
            (favorites::dsl::username.eq(designated_username))))
            .into_boxed();

        match latest_pub {
            OrderOption::Title=> {
                use crate::dbconfig::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => {
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(date_of_recording.asc());

                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(date_of_recording.desc());
                    }
                }
            }
        }

        if title.is_some() {
            use crate::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query
                .filter(podcasttitle.like(format!("%{}%", title.unwrap())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Favorite)>(conn).expect("Error loading podcasts");
        let distinct_podcasts:Vec<(Podcast, Favorite)> = pr.iter()
            .filter(|c|{
                if matching_podcast_ids.contains(&c.0.id){
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            }).map(|c|{
            (c.clone().0, c.clone().2)
        }).collect::<Vec<(Podcast, Favorite)>>();
        distinct_podcasts
    }


    pub fn search_podcasts(conn: &mut DbConnection, order: OrderCriteria, title: Option<String>,
                           latest_pub: OrderOption,
                           designated_username: String) -> Vec<(Podcast, Option<Favorite>)> {
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::id as podcastsid;
        use crate::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::dbconfig::schema::favorites::dsl::username as f_username;

        let mut query = podcasts.inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .left_join(f_db.on(f_username.eq(designated_username).and(f_id.eq(podcast_id))))
            .into_boxed();

        match latest_pub {
            OrderOption::Title=> {
                use crate::dbconfig::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => {
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(date_of_recording.asc());

                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(date_of_recording.desc());
                    }
                }
            }
        }

        if title.is_some() {
            use crate::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query
                .filter(podcasttitle.like(format!("%{}%", title.unwrap())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Option<Favorite>)>(conn).expect("Error loading \
            podcasts");
        let distinct_podcasts = pr.iter()
            .filter(|c|{
                if matching_podcast_ids.contains(&c.0.id){
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            }).map(|c|{
            (c.clone().0, c.clone().2)
        }).collect::<Vec<(Podcast, Option<Favorite>)>>();
        distinct_podcasts
    }
}