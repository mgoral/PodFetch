use crate::dbconfig::schema::*;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel::{OptionalExtension, RunQueryDsl};
use crate::service::environment_service::OidcConfig;
use utoipa::ToSchema;
use crate::DbConnection;
use crate::utils::do_retry::do_retry;
use diesel::insert_into;
use crate::constants::constants::DEFAULT_SETTINGS;

#[derive(
    Serialize, Deserialize, Queryable, Insertable, Debug, Clone, Identifiable, AsChangeset,ToSchema
)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigModel {
    pub podindex_configured: bool,
    pub rss_feed: String,
    pub server_url: String,
    pub basic_auth: bool,
    pub oidc_configured: bool,
    pub oidc_config: Option<OidcConfig>
}


impl Setting{

    pub fn get_settings(conn: &mut DbConnection) -> Option<Setting> {
        use crate::dbconfig::schema::settings::dsl::*;

        settings
            .first::<Setting>(conn)
            .optional()
            .unwrap()
    }

    pub fn update_settings(setting: Setting, conn:&mut DbConnection) -> Setting {
        use crate::dbconfig::schema::settings::dsl::*;
        let setting_to_update = settings
            .first::<Setting>(conn)
            .expect("Error loading settings");
        do_retry(||{diesel::update(&setting_to_update)
            .set(setting.clone())
            .get_result::<Setting>(conn)})
            .expect("Error updating settings")
    }

    pub fn insert_default_settings(conn: &mut DbConnection) {
        use crate::dbconfig::schema::settings::dsl::*;
        use diesel::ExpressionMethods;
        do_retry(||{insert_into(settings)
            .values((
                id.eq(1),
                auto_download.eq(DEFAULT_SETTINGS.auto_download),
                auto_cleanup.eq(DEFAULT_SETTINGS.auto_cleanup),
                auto_cleanup_days.eq(DEFAULT_SETTINGS.auto_cleanup_days),
                podcast_prefill.eq(DEFAULT_SETTINGS.podcast_prefill))
            )
            .execute(conn)})
            .expect("Error setting default values");
    }
}
