use crate::{
    domain::planning::Mesocycle, dto::planning::MesocycleDTO
};
use flutter_rust_bridge::frb;

//Fake it until there is a solution to store mesocycles
#[frb(sync)]
pub fn get_mesocycles() -> Vec<MesocycleDTO> {
    vec![
        MesocycleDTO { 
            id: None,
            name: String::from("Strength Block"),
            microcycles: Vec::new()
        },
        MesocycleDTO { 
            id: None,
            name: String::from("Hypertrophy Block"),
            microcycles: Vec::new() 
        },
    ]
}

#[frb(sync)]
pub fn create_mesocycle(name: String) -> MesocycleDTO {
    let mesocycle = Mesocycle::new(name);
    MesocycleDTO::from(&mesocycle)
}