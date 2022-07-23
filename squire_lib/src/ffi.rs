use crate::{
    error::TournamentError,
    fluid_pairings::FluidPairings,
    operations::TournOp,
    player::{Player, PlayerId},
    player_registry::{PlayerIdentifier, PlayerRegistry},
    round::{Round, RoundId, RoundResult, RoundStatus},
    round_registry::{RoundIdentifier, RoundRegistry},
    scoring::{Score, Standings},
    settings::{
        self, FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
        SwissPairingsSetting, TournamentSetting,
    },
    standard_scoring::{StandardScore, StandardScoring},
    swiss_pairings::SwissPairings,
    tournament::{Tournament, TournamentPreset},
};

use mtgjson::model::deck::Deck;

use libc::c_char;
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    hash::{Hash, Hasher},
    time::Duration,
};

lazy_static! {
    /// A map of tournament ids to tournaments
    /// this is used for allocating ffi tournaments
    /// all ffi tournaments are always deeply copied
    /// at the lanuage barrier
    static ref ffi_tournament_registry: HashMap<String, Tournament> = HashMap::new();
}

