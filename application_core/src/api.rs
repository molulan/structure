use crate::Mesocycle;
use flutter_rust_bridge::frb;

//Fake it until there is a solution to store mesocycles
#[frb(sync)]
pub fn get_mesocycles() -> Vec<Mesocycle> {
    vec![
        Mesocycle::new("Strength Block"),
        Mesocycle::new("Hypertrophy Block"),
    ]
}

#[frb(sync)]
pub fn create_mesocycle(name: String) -> Mesocycle {
    Mesocycle::new(&name)
}
