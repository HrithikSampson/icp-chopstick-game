use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::caller;
use ic_cdk_macros::{init, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};
//use crate::rand::generate_uuid_v4;

use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpMethod};

pub mod rand;
#[derive(CandidType, Deserialize, Debug, Clone)]
struct Player {
    id: Principal,
    left_hand: u8,
    right_hand: u8,
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Clone)]
enum GameState {
    WaitingForPlayer,
    InProgress,
    Finished(Principal),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
enum Turn {
    Player1,
    Player2,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
struct Game {
    session_id: String,
    player1: Player,
    player2: Option<Player>,
    state: GameState,
    current_turn: Turn,
}

#[derive(Debug,Default, CandidType, Deserialize)]
struct ChopsticksGameService {
    games: HashMap<String, Game>,

}

impl Storable for ChopsticksGameService {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded { max_size: 10000, is_fixed_size: false };
}


impl Storable for Game {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded { max_size: 10000, is_fixed_size: false };
}



thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static GAME_SERVICE: RefCell<StableBTreeMap<u8, ChopsticksGameService, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

const SERVICE_ID: u8 = 0;
#[init]
fn init() {
    let init_state = ChopsticksGameService {
        games: HashMap::new(),
    };
    GAME_SERVICE.with(|service| {
        service.borrow_mut().insert(SERVICE_ID, init_state);
    });
}

impl Game {
    async fn new() -> Self {
        let temp = 0;
        let address_temp = &temp as *const i32;
        let rng = address_temp as i32;
        let choice = if rng%2 == 0 { false} else{ true };
        
        let url = "https://www.uuidtools.com/api/generate/v1".to_string();
        let request_headers = vec![];
        let request = CanisterHttpRequestArgument {
            url,
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: None,
            transform: None,
            headers: request_headers,
        };
        let mut uuid = None;
        match http_request(request, 1_603_076_400).await {
            Ok((response,)) => {
                ic_cdk::println!("{:?}",response.body);
                if response.status == 200 as u32 {
                    let uuid_array= 
                        serde_json::from_slice::<Vec<String>>(&response.body).expect("Failed to parse JSON response");
                    ic_cdk::println!("urls:{:?}",uuid_array);
                    //uuid = Some(uuid_name.clone().to_string());
                    if let Some(uuid_name) = uuid_array.get(0) {
                        uuid = Some(uuid_name.clone().to_string());
                    }
                }
            }
            Err((code, message)) => {
                ic_cdk::println!(
                    "The http_request resulted in an error. Code: {:?}, Message: {}",
                    code, message
                );
            }
        };
        let s = &(*uuid.unwrap()).to_string();
        Game {
            session_id: s.to_string(),
            player1: Player { id: caller(), left_hand: 1, right_hand: 1 },
            player2: None,
            state: GameState::WaitingForPlayer,
            current_turn: if choice { Turn::Player1 } else { Turn::Player2 },
        }
    }

    fn join(&mut self, player: Player) {
        if self.state == GameState::WaitingForPlayer && self.player2.is_none() {
            self.player2 = Some(player);
            self.state = GameState::InProgress;
        }
    }

    fn make_move_opponent(&mut self, player_id: Principal, hand: u8, target_hand: u8) -> Result<(),String>{
        if self.state != GameState::InProgress {
            return Err(String::from("Game is not in Progress"));
        }

        // Determine if it's player1's or player2's turn and if the move is valid
        let (active_player, opponent) = match self.current_turn {
            Turn::Player1 if self.player1.id == player_id => (&mut self.player1, self.player2.as_mut()),
            Turn::Player2 if self.player2.as_ref().map_or(false, |p| p.id == player_id) => (self.player2.as_mut().unwrap(), Some(&mut self.player1)),
            _ => return Err(String::from("Player not allowed to play")), // Not the player's turn or player not found
        };

        // hand and target_hand are 0 for left hand and 1 for right hand, adjust as needed
        let active_hand = if hand == 0 { active_player.left_hand } else { active_player.right_hand };
        if active_hand == 0 { return Err(String::from("Cannot make move with this hand")); } // Cannot make a move with an inactive hand

        if let Some(opponent) = opponent {
            let opponent_hand = if target_hand == 0 { &mut opponent.left_hand } else { &mut opponent.right_hand };
            *opponent_hand += active_hand;
            if *opponent_hand >= 5 { *opponent_hand = 0; } // Reset hand if it reaches the threshold

            // Check if the game has ended
            if opponent.left_hand == 0 && opponent.right_hand == 0 {
                self.state = GameState::Finished(player_id);
            } else {
                // Switch turns
                self.current_turn = match self.current_turn {
                    Turn::Player1 => Turn::Player2,
                    Turn::Player2 => Turn::Player1,
                };
            }
        }
        Ok(())
    }
    
    fn make_move_other_hand(&mut self, player_id: Principal, current_hand: u8, transfer: u8) -> Result<(),String>{
        if self.state != GameState::InProgress {
            return Err(String::from("Game is not in Progress"));
        }

        // Determine if it's player1's or player2's turn and if the move is valid
        let (active_player, _opponent) = match self.current_turn {
            Turn::Player1 if self.player1.id == player_id => (&mut self.player1, self.player2.as_mut()),
            Turn::Player2 if self.player2.as_ref().map_or(false, |p| p.id == player_id) => (self.player2.as_mut().unwrap(), Some(&mut self.player1)),
            _ => return Err(String::from("You are not allowed to play")), // Not the player's turn or player not found
        };
        let left = &mut active_player.left_hand;
        let right = &mut active_player.right_hand;
        // hand and target_hand are 0 for left hand and 1 for right hand, adjust as needed
        let active_hand;
        let target_hand;
        if current_hand == 0 {
            active_hand = left;
            target_hand = right;
        } else {
            active_hand = right;
            target_hand = left;
        }
        if transfer>*active_hand {
            return Err(String::from("You cant transfer more than the amount in ur hand"));
        }
        let new_active_hand = *active_hand - transfer;
        let new_target_hand = (*target_hand + transfer)%(5 as u8);
        
        //Simulate Transfer
        if (new_active_hand == *target_hand) && (new_target_hand == *active_hand) {
            // makes the game interesting
            return Err(String::from("Symmetric Operations are not allowed"));
        }
        *active_hand = new_active_hand;
        *target_hand = new_target_hand;
        
        Ok(())
    }
}

#[update]
async fn start_game() -> Result<String, String> {
    let game = Game::new().await;
    let session_id = game.session_id.clone();
    GAME_SERVICE.with(|service| {
        let games = service.borrow_mut().get(&SERVICE_ID);

        if let Some(mut game_service) = games {
            
            game_service.games.insert(session_id.clone(), game);
            ic_cdk::println!("{:?}",game_service);
            service.borrow_mut().insert(SERVICE_ID, game_service);
        }
        else{

        }
        ;
    });
    Ok(session_id)
}

#[update]
fn join_game(session_id: String) -> Result<(), String> {
    let player = Player { id: caller() , left_hand:1 ,right_hand: 1};
    GAME_SERVICE.with(|service| {
        let games = service.borrow_mut().get(&SERVICE_ID);
        if let Some(mut game_service) = games {
            ic_cdk::println!("{:?}",game_service);
            if let Some(game) = game_service.games.get_mut(&session_id) {
                game.join(player);
                service.borrow_mut().insert(SERVICE_ID, game_service);
                Ok(())
            } else {
                Err("Game not found".to_string())
            }
        }
        else {
            Ok(())
        }
    })
}

#[update]
fn make_move_opponent(session_id: String, hand: u8, target_hand: u8) -> Result<(), String> {
    let player_id = caller();
    GAME_SERVICE.with(|service| {
        
        let games = service.borrow_mut().get(&SERVICE_ID);
        if let Some(mut game_service) = games {
            let ret = if let Some(mut game) = game_service.games.get_mut(&session_id) {
                match game.make_move_opponent(player_id, hand, target_hand) {
                    Result::Ok(_code) => Ok(()),
                    Result::Err(str) => Err(str)
                }
            } else {
                Err("Game not found".to_string())
            };
            
            service.borrow_mut().insert(SERVICE_ID, game_service);
            return ret;
        }
        else {
            Ok(())
        }
    })
}

#[update]
fn make_move_other_hand(session_id: String,current_hand: u8, transfer: u8) -> Result<(), String> {
    let player_id = caller();
    GAME_SERVICE.with(|service| {
        
        let mut games = service.borrow_mut().get(&SERVICE_ID);
        if let Some(mut game_service) = games {
            let ret = if let Some(mut game) = game_service.games.get_mut(&session_id) {
                match game.make_move_other_hand(player_id, current_hand, transfer) {
                    Result::Ok(_code) => Ok(()),
                    Result::Err(str) => Err(str)
                }
            } else {
                Err("Game not found".to_string())
            };
            service.borrow_mut().insert(SERVICE_ID, game_service);
            ret
        }
        else {
            Ok(())
        }
    })
}

#[query]
fn get_game_state(session_id: String) -> Result<Game, String> {
    GAME_SERVICE.with(|service| {
        let games = service.borrow();
        if let Some(game_service) = games.get(&SERVICE_ID) {
            ic_cdk::println!("game_service:{:?}",game_service);
            if let Some(game) = game_service.games.get(&session_id){
                Ok(game.clone())
            }
            else{
                Err("Game not found".to_string())
            }
        } else {
            Err("Game not found".to_string())
        }
    })
}

// Export the candid interface
ic_cdk_macros::export_candid!();
