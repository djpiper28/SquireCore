#![allow(unused)]
use dashmap::DashMap;
use rocket::{routes, Build, Rocket};
//use squire_sdk::accounts::{AccountId, UserAccount};
//use uuid::Uuid;

#[cfg(test)]
mod tests;

//mod accounts;
mod matches;
mod players;
mod tournaments;

//use accounts::*;
use players::*;
use tournaments::*;

pub fn init() -> Rocket<Build> {
    //let _ = USERS_MAP.set(DashMap::new());
    //let _ = ORGS_MAP.set(DashMap::new());
    let _ = TOURNS_MAP.set(DashMap::new());
    rocket::build()
        //.mount("/api/v1/accounts", routes![users, all_users, orgs])
        .mount(
            "/api/v1/tournaments",
            routes![
                create_tournament,
                get_tournament,
                get_all_tournaments,
                get_standings,
                slice_ops,
                sync,
                rollback,
                get_player,
                get_all_players,
                get_active_players,
                get_player_count,
                get_active_player_count,
                get_player_deck,
                get_all_decks,
                get_all_player_decks,
                get_player_matches,
                get_latest_player_match,
            ],
        )
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let client = init();
    /*
    let id = AccountId(Uuid::new_v4());
    let account = UserAccount {
        external_id: id.clone(),
        display_name: "Tyler Bloom".to_string(),
        account_name: "TylerBloom".to_string(),
    };
    println!("{account:?}");
    USERS_MAP.get().unwrap().insert(id, account);
    */
    client.launch().await?;

    Ok(())
}
