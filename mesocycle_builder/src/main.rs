use application_core::{Exercise, Mesocycle, Microcycle, Workout};
use std::io::{self, Write};

fn main() {
    println!("====== Mesocycle Builder ======");

    let mesocycle_name = prompt("Enter mesocycle name: ");
    let mut mesocycle = Mesocycle::new(&mesocycle_name);

    mesocycle = add_microcycles(mesocycle);

    dbg!(&mesocycle);
}

fn prompt(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

fn add_microcycles(mut mesocycle: Mesocycle) -> Mesocycle {
    let mut micro_count = 0;
    loop {
        match prompt(&format!(
            "Type '+' to add a microcycle {}, leave blank to finalize mesocycle: ",
            micro_count + 1
        ))
        .as_str()
        {
            "+" => {
                micro_count += 1;
                let mut microcycle = Microcycle::new(&format!("{micro_count}"));
                microcycle = add_workouts(microcycle);
                mesocycle.add_microcycle(microcycle)
            }
            "" => break,
            _ => println!("Invalid input"),
        }
    }
    mesocycle
}

fn add_workouts(mut microcycle: Microcycle) -> Microcycle {
    let mut workout_count = 0;
    loop {
        match prompt(&format!(
            "Enter workout name to add workout number {} (leave blank to finalize microcycle)",
            workout_count + 1
        ))
        .as_str()
        {
            "" => break,
            workout_name => {
                workout_count += 1;
                let mut workout = Workout::new(workout_name);
                workout = add_exercises(workout);
                microcycle.add_workout(workout)
            }
        }
    }
    microcycle
}

fn add_exercises(mut workout: Workout) -> Workout {
    let mut exercise_count = 0;
    loop {
        match prompt(&format!(
            "Enter exercise name to add exeercise number {} (leave blank to finalize workout)",
            exercise_count + 1
        ))
        .as_str()
        {
            "" => break,
            exercise_name => {
                exercise_count += 1;
                let exercise = Exercise::bodyweight(exercise_name);
                workout.add_exercise(exercise)
            }
        }
    }
    workout
}
