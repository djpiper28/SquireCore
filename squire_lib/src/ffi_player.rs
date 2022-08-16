use crate::ffi::{clone_string_to_c_string, FFI_TOURNAMENT_REGISTRY};
use crate::{
    identifiers::{PlayerId, TournamentId},
    player::{Player, PlayerStatus},
    tournament::Tournament,
};
use std::option::Option;
use std::os::raw::c_char;

impl PlayerId {
    /// Returns the player if it can be found in the tournament
    fn get_tourn_player(self, tid: TournamentId) -> Option<Player> {
        let tourn: Tournament;
        unsafe {
            match FFI_TOURNAMENT_REGISTRY.get_mut().unwrap().get(&tid) {
                Some(t) => tourn = t.value().clone(),
                None => {
                    println!("Cannot find player in get_tourn_player();");
                    return None;
                }
            }
        }

        tourn.player_reg.get_player(&self.into()).cloned()
    }

    /// Returns the player name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_name(self, tid: TournamentId) -> *const c_char {
        let player: Player;
        match self.get_tourn_player(tid) {
            Some(p) => player = p,
            None => {
                return std::ptr::null();
            }
        }

        return clone_string_to_c_string(player.name);
    }

    /// Returns the player's game name if they can be found
    /// NULL is returned on error or, failure to find
    #[no_mangle]
    pub extern "C" fn pid_game_name(self, tid: TournamentId) -> *const c_char {
        let player: Player;
        match self.get_tourn_player(tid) {
            Some(p) => player = p,
            None => {
                return std::ptr::null();
            }
        }

        match player.game_name {
            Some(n) => {
                return clone_string_to_c_string(n);
            }
            None => {
                return std::ptr::null();
            }
        }
    }

    /// Returns the player's status if they can be found
    /// Dropped on error.
    #[no_mangle]
    pub extern "C" fn pid_status(self, tid: TournamentId) -> PlayerStatus {
        let player: Player;
        match self.get_tourn_player(tid) {
            Some(p) => player = p,
            None => {
                return PlayerStatus::Dropped;
            }
        }

        return player.status;
    }
}
