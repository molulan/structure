use crate::domain::planning::{
    ExerciseType, Intensity, PercentOneRepMax, RepTarget, Rir, Rpe, SetGroup,
    SetGroupValidationError, SetStyle, Weight, WeightUnit,
};
use crate::persistence::library_exercises::exercise_type_from_str;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, thiserror::Error)]
pub enum SetGroupError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated planned exercise {id} not found")]
    AssociatedPlannedExerciseNotFound { id: i64 },
    #[error("set group {id} not found")]
    NotFound { id: i64 },
    #[error("reorder list does not match the set groups of planned exercise {planned_exercise_id}")]
    ReorderMismatch { planned_exercise_id: i64 },
    #[error(transparent)]
    Invalid(#[from] SetGroupValidationError),
}

pub(super) fn create_set_groups_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS set_groups (
            id INTEGER PRIMARY KEY,
            planned_exercise_id INTEGER NOT NULL REFERENCES planned_exercises(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            set_style TEXT NOT NULL CHECK(
                set_style IN ('Regular', 'Myorep', 'MyorepMatch', 'Drop')
            ),
            number_of_sets INTEGER NOT NULL CHECK(number_of_sets > 0),
            rep_min INTEGER NOT NULL CHECK(rep_min > 0),
            rep_max INTEGER CHECK(rep_max IS NULL OR rep_max > rep_min),
            intensity_type TEXT NOT NULL CHECK(
                intensity_type IN ('Rir', 'Rpe', 'PercentOneRepMax', 'TargetWeight', 'WeightIncrement')
            ),
            intensity_value REAL NOT NULL,
            intensity_weight_unit TEXT CHECK(intensity_weight_unit IN ('Kg', 'Lbs')),
            UNIQUE(planned_exercise_id, position),
            CHECK(
                (intensity_type IN ('TargetWeight', 'WeightIncrement'))
                    = (intensity_weight_unit IS NOT NULL)
            )
        )",
        [],
    )?;
    Ok(())
}

fn planned_exercise_type(
    conn: &Connection,
    planned_exercise_id: i64,
) -> rusqlite::Result<Option<ExerciseType>> {
    conn.query_row(
        "SELECT le.exercise_type FROM planned_exercises pe
         JOIN library_exercises le ON le.id = pe.library_exercise_id
         WHERE pe.id = ?1",
        [planned_exercise_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|name| name.map(|name| exercise_type_from_str(&name)))
}

fn set_group_exercise_type(
    conn: &Connection,
    set_group_id: i64,
) -> rusqlite::Result<Option<ExerciseType>> {
    conn.query_row(
        "SELECT le.exercise_type FROM set_groups sg
         JOIN planned_exercises pe ON pe.id = sg.planned_exercise_id
         JOIN library_exercises le ON le.id = pe.library_exercise_id
         WHERE sg.id = ?1",
        [set_group_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|name| name.map(|name| exercise_type_from_str(&name)))
}

pub fn create(
    conn: &mut Connection,
    planned_exercise_id: i64,
    style: SetStyle,
    number_of_sets: u32,
    reps: RepTarget,
    intensity: Intensity,
) -> Result<SetGroup, SetGroupError> {
    let tx = conn.transaction()?;

    let Some(exercise_type) = planned_exercise_type(&tx, planned_exercise_id)? else {
        return Err(SetGroupError::AssociatedPlannedExerciseNotFound {
            id: planned_exercise_id,
        });
    };

    let next_position: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM set_groups WHERE planned_exercise_id = ?1",
        [planned_exercise_id],
        |row| row.get(0),
    )?;
    let position = u32::try_from(next_position)
        .expect("positions are non-negative and no exercise will have 4 billion set groups");

    let (rep_min, rep_max) = encode_reps(reps);
    let (intensity_type, intensity_value, intensity_weight_unit) = encode_intensity(intensity);

    tx.execute(
        "INSERT INTO set_groups
            (planned_exercise_id, position, set_style, number_of_sets,
             rep_min, rep_max, intensity_type, intensity_value, intensity_weight_unit)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            planned_exercise_id,
            position,
            style.to_string(),
            number_of_sets,
            rep_min,
            rep_max,
            intensity_type,
            intensity_value,
            intensity_weight_unit,
        ],
    )?;

    let id = tx.last_insert_rowid();
    let set_group = SetGroup::new(
        id,
        position,
        exercise_type,
        style,
        number_of_sets,
        reps,
        intensity,
    )?;
    tx.commit()?;
    Ok(set_group)
}

pub fn update(
    conn: &mut Connection,
    id: i64,
    style: SetStyle,
    number_of_sets: u32,
    reps: RepTarget,
    intensity: Intensity,
) -> Result<SetGroup, SetGroupError> {
    let tx = conn.transaction()?;

    let Some(exercise_type) = set_group_exercise_type(&tx, id)? else {
        return Err(SetGroupError::NotFound { id });
    };

    let (rep_min, rep_max) = encode_reps(reps);
    let (intensity_type, intensity_value, intensity_weight_unit) = encode_intensity(intensity);

    let position: Option<i64> = tx
        .query_row(
            "UPDATE set_groups SET
                set_style = ?1, number_of_sets = ?2, rep_min = ?3, rep_max = ?4,
                intensity_type = ?5, intensity_value = ?6, intensity_weight_unit = ?7
             WHERE id = ?8
             RETURNING position",
            params![
                style.to_string(),
                number_of_sets,
                rep_min,
                rep_max,
                intensity_type,
                intensity_value,
                intensity_weight_unit,
                id,
            ],
            |row| row.get(0),
        )
        .optional()?;

    let position = match position {
        Some(position) => {
            u32::try_from(position).expect("position stored in DB was originally a u32")
        }
        None => return Err(SetGroupError::NotFound { id }),
    };

    let set_group = SetGroup::new(
        id,
        position,
        exercise_type,
        style,
        number_of_sets,
        reps,
        intensity,
    )?;
    tx.commit()?;
    Ok(set_group)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), SetGroupError> {
    let deleted = conn.execute("DELETE FROM set_groups WHERE id = ?1", [id])?;

    if deleted == 0 {
        return Err(SetGroupError::NotFound { id });
    }

    Ok(())
}

pub fn reorder(
    conn: &mut Connection,
    planned_exercise_id: i64,
    ordered_ids: &[i64],
) -> Result<(), SetGroupError> {
    let matched = super::positions::reorder(
        conn,
        "set_groups",
        "planned_exercise_id",
        planned_exercise_id,
        ordered_ids,
    )?;

    if matched {
        Ok(())
    } else {
        Err(SetGroupError::ReorderMismatch {
            planned_exercise_id,
        })
    }
}

fn row_to_set_group(row: &rusqlite::Row<'_>) -> rusqlite::Result<SetGroup> {
    let id = row.get(0)?;
    let position: i64 = row.get(1)?;
    let position = u32::try_from(position).expect("position stored in DB was originally a u32");
    let set_style: String = row.get(2)?;
    let number_of_sets: i64 = row.get(3)?;
    let number_of_sets =
        u32::try_from(number_of_sets).expect("number_of_sets stored in DB was originally a u32");
    let rep_min: i64 = row.get(4)?;
    let rep_max: Option<i64> = row.get(5)?;
    let intensity_type: String = row.get(6)?;
    let intensity_value: f64 = row.get(7)?;
    let intensity_weight_unit: Option<String> = row.get(8)?;

    let style = set_style_from_str(&set_style);
    let reps = decode_reps(rep_min, rep_max);
    let intensity = decode_intensity(
        &intensity_type,
        intensity_value,
        intensity_weight_unit.as_deref(),
    );

    Ok(SetGroup::new_unchecked(
        id,
        position,
        style,
        number_of_sets,
        reps,
        intensity,
    ))
}

pub fn list(conn: &Connection, planned_exercise_id: i64) -> Result<Vec<SetGroup>, SetGroupError> {
    if planned_exercise_type(conn, planned_exercise_id)?.is_none() {
        return Err(SetGroupError::AssociatedPlannedExerciseNotFound {
            id: planned_exercise_id,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT id, position, set_style, number_of_sets, rep_min, rep_max,
                intensity_type, intensity_value, intensity_weight_unit
         FROM set_groups
         WHERE planned_exercise_id = ?1
         ORDER BY position ASC",
    )?;

    let rows = stmt.query_map([planned_exercise_id], row_to_set_group)?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }
    Ok(groups)
}

fn encode_reps(reps: RepTarget) -> (i64, Option<i64>) {
    match reps {
        RepTarget::Exact(count) => (count.value() as i64, None),
        RepTarget::Range(range) => (range.min() as i64, Some(range.max() as i64)),
    }
}

fn decode_reps(rep_min: i64, rep_max: Option<i64>) -> RepTarget {
    let min = u32::try_from(rep_min).expect("rep_min out of u32 range");
    match rep_max {
        None => RepTarget::exact(min).expect("rep count stored in DB was validated on write"),
        Some(max) => {
            let max = u32::try_from(max).expect("rep_max out of u32 range");
            RepTarget::range(min, max).expect("rep range stored in DB was validated on write")
        }
    }
}

fn encode_intensity(intensity: Intensity) -> (&'static str, f64, Option<&'static str>) {
    match intensity {
        Intensity::Rir(rir) => ("Rir", rir.value() as f64, None),
        Intensity::Rpe(rpe) => ("Rpe", rpe.value() as f64, None),
        Intensity::PercentOneRepMax(pct) => ("PercentOneRepMax", pct.value() as f64, None),
        Intensity::TargetWeight(weight) => (
            "TargetWeight",
            weight.value(),
            Some(weight_unit_to_str(weight.unit())),
        ),
        Intensity::WeightIncrement(weight) => (
            "WeightIncrement",
            weight.value(),
            Some(weight_unit_to_str(weight.unit())),
        ),
    }
}

fn decode_intensity(intensity_type: &str, value: f64, unit: Option<&str>) -> Intensity {
    match intensity_type {
        "Rir" => Intensity::Rir(Rir::new(value as i8).expect("invalid Rir value in DB")),
        "Rpe" => Intensity::Rpe(Rpe::new(value as u8).expect("invalid Rpe value in DB")),
        "PercentOneRepMax" => Intensity::PercentOneRepMax(
            PercentOneRepMax::new(value as u8).expect("invalid PercentOneRepMax value in DB"),
        ),
        "TargetWeight" => Intensity::TargetWeight(decode_weight(value, unit)),
        "WeightIncrement" => Intensity::WeightIncrement(decode_weight(value, unit)),
        other => panic!("unknown intensity_type in DB: {other}"),
    }
}

fn decode_weight(value: f64, unit: Option<&str>) -> Weight {
    let unit = unit.expect("weight intensity has no weight unit in DB");
    Weight::new(value, weight_unit_from_str(unit))
}

fn set_style_from_str(s: &str) -> SetStyle {
    match s {
        "Regular" => SetStyle::Regular,
        "Myorep" => SetStyle::Myorep,
        "MyorepMatch" => SetStyle::MyorepMatch,
        "Drop" => SetStyle::Drop,
        other => panic!("unknown set_style in DB: {other}"),
    }
}

fn weight_unit_to_str(unit: WeightUnit) -> &'static str {
    match unit {
        WeightUnit::Kg => "Kg",
        WeightUnit::Lbs => "Lbs",
    }
}

fn weight_unit_from_str(s: &str) -> WeightUnit {
    match s {
        "Kg" => WeightUnit::Kg,
        "Lbs" => WeightUnit::Lbs,
        other => panic!("unknown weight_unit in DB: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::{MesocycleMode, PlannedExercise},
        persistence::{
            connection, library_exercises, mesocycles, microcycles, planned_exercises, workouts,
        },
    };

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("failed to create test database")
    }

    fn planned_exercise_of_type(conn: &Connection, exercise_type: ExerciseType) -> PlannedExercise {
        let mesocycle = mesocycles::create(conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let microcycle =
            microcycles::create(conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = workouts::create(conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed");
        let exercise = library_exercises::create(conn, "Bench Press", exercise_type)
            .expect("exercise creation should succeed");
        planned_exercises::create(conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed")
    }

    fn weighted_planned_exercise(conn: &Connection) -> PlannedExercise {
        planned_exercise_of_type(conn, ExerciseType::Weighted)
    }

    #[test]
    fn create_set_group_on_existing_planned_exercise_succeeds() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = create(
            &mut conn,
            planned.id(),
            SetStyle::Regular,
            3,
            RepTarget::range(8, 12).unwrap(),
            Intensity::Rir(Rir::new(2).unwrap()),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn create_set_group_for_nonexistent_planned_exercise_returns_error() {
        let mut conn = setup_test_db();

        let result = create(
            &mut conn,
            9999,
            SetStyle::Regular,
            3,
            RepTarget::exact(10).unwrap(),
            Intensity::Rpe(Rpe::new(8).unwrap()),
        );

        assert!(matches!(
            result,
            Err(SetGroupError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn create_set_group_with_proximity_intensity_on_failure_style_returns_invalid() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = create(
            &mut conn,
            planned.id(),
            SetStyle::Myorep,
            3,
            RepTarget::exact(10).unwrap(),
            Intensity::Rir(Rir::new(1).unwrap()),
        );

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(
                SetGroupValidationError::IntensityIncompatibleWithStyle { .. }
            ))
        ));
    }

    #[test]
    fn create_set_group_with_weight_intensity_on_bodyweight_returns_invalid() {
        let mut conn = setup_test_db();
        let planned = planned_exercise_of_type(&conn, ExerciseType::Bodyweight);

        let result = create(
            &mut conn,
            planned.id(),
            SetStyle::Regular,
            3,
            RepTarget::exact(10).unwrap(),
            Intensity::TargetWeight(Weight::new(20.0, WeightUnit::Kg)),
        );

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(
                SetGroupValidationError::WeightIntensityOnBodyweight { .. }
            ))
        ));
    }

    #[test]
    fn set_groups_get_sequential_positions() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        let new_group = |conn: &mut Connection| {
            create(
                conn,
                planned.id(),
                SetStyle::Regular,
                3,
                RepTarget::exact(5).unwrap(),
                Intensity::Rir(Rir::new(2).unwrap()),
            )
            .expect("set group creation should succeed")
        };

        assert_eq!(new_group(&mut conn).position(), 0);
        assert_eq!(new_group(&mut conn).position(), 1);
        assert_eq!(new_group(&mut conn).position(), 2);
    }

    #[test]
    fn list_set_groups_returns_empty_for_planned_exercise_with_none() {
        let conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result =
            list(&conn, planned.id()).expect("listing for an existing planned exercise succeeds");

        assert!(result.is_empty());
    }

    #[test]
    fn list_set_groups_returns_error_when_planned_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = list(&conn, 9999);

        assert!(matches!(
            result,
            Err(SetGroupError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn every_intensity_and_rep_target_round_trips_through_the_database() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let created: Vec<SetGroup> = [
            (
                RepTarget::exact(5).unwrap(),
                Intensity::Rir(Rir::new(0).unwrap()),
            ),
            (
                RepTarget::range(8, 12).unwrap(),
                Intensity::Rpe(Rpe::new(9).unwrap()),
            ),
            (
                RepTarget::exact(3).unwrap(),
                Intensity::PercentOneRepMax(PercentOneRepMax::new(85).unwrap()),
            ),
            (
                RepTarget::range(6, 10).unwrap(),
                Intensity::TargetWeight(Weight::new(100.0, WeightUnit::Kg)),
            ),
            (
                RepTarget::exact(8).unwrap(),
                Intensity::WeightIncrement(Weight::new(-2.5, WeightUnit::Lbs)),
            ),
        ]
        .into_iter()
        .map(|(reps, intensity)| {
            create(
                &mut conn,
                planned.id(),
                SetStyle::Regular,
                4,
                reps,
                intensity,
            )
            .expect("set group creation should succeed")
        })
        .collect();

        let listed = list(&conn, planned.id()).expect("listing should succeed");

        assert_eq!(listed, created);
    }

    #[test]
    fn update_set_group_changes_its_prescription_and_keeps_position() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        let group = create(
            &mut conn,
            planned.id(),
            SetStyle::Regular,
            3,
            RepTarget::exact(5).unwrap(),
            Intensity::Rir(Rir::new(3).unwrap()),
        )
        .expect("set group creation should succeed");

        let updated = update(
            &mut conn,
            group.id(),
            SetStyle::Myorep,
            2,
            RepTarget::range(10, 20).unwrap(),
            Intensity::TargetWeight(Weight::new(40.0, WeightUnit::Kg)),
        )
        .expect("update should succeed");

        assert_eq!(updated.style(), SetStyle::Myorep);
        assert_eq!(updated.number_of_sets(), 2);
        assert_eq!(updated.reps(), RepTarget::range(10, 20).unwrap());
        assert_eq!(
            updated.intensity(),
            Intensity::TargetWeight(Weight::new(40.0, WeightUnit::Kg))
        );
        assert_eq!(updated.position(), group.position());

        let listed = list(&conn, planned.id()).expect("listing should succeed");
        assert_eq!(listed, vec![updated]);
    }

    #[test]
    fn update_set_group_returns_not_found_when_it_does_not_exist() {
        let mut conn = setup_test_db();

        let result = update(
            &mut conn,
            9999,
            SetStyle::Regular,
            3,
            RepTarget::exact(5).unwrap(),
            Intensity::Rir(Rir::new(2).unwrap()),
        );

        assert!(matches!(result, Err(SetGroupError::NotFound { id: 9999 })));
    }

    /// Returns the planned exercise id and three of its set groups.
    fn planned_exercise_with_three_groups(
        conn: &mut Connection,
    ) -> (i64, SetGroup, SetGroup, SetGroup) {
        let planned = weighted_planned_exercise(conn);
        let new_group = |conn: &mut Connection| {
            create(
                conn,
                planned.id(),
                SetStyle::Regular,
                3,
                RepTarget::exact(5).unwrap(),
                Intensity::Rir(Rir::new(2).unwrap()),
            )
            .expect("set group creation should succeed")
        };
        (
            planned.id(),
            new_group(conn),
            new_group(conn),
            new_group(conn),
        )
    }

    #[test]
    fn create_set_group_after_delete_does_not_reuse_a_position() {
        let mut conn = setup_test_db();
        let (planned_id, _a, middle, _c) = planned_exercise_with_three_groups(&mut conn);

        delete(&conn, middle.id()).expect("delete should succeed");

        let next = create(
            &mut conn,
            planned_id,
            SetStyle::Regular,
            3,
            RepTarget::exact(5).unwrap(),
            Intensity::Rir(Rir::new(2).unwrap()),
        )
        .expect("set group creation should succeed");
        assert_eq!(next.position(), 3);
    }

    #[test]
    fn delete_set_group_removes_it() {
        let mut conn = setup_test_db();
        let (planned_id, group, _b, _c) = planned_exercise_with_three_groups(&mut conn);

        delete(&conn, group.id()).expect("delete should succeed");

        let listed = list(&conn, planned_id).expect("listing should succeed");
        assert!(!listed.iter().any(|g| g.id() == group.id()));
    }

    #[test]
    fn delete_set_group_returns_not_found_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 9999);

        assert!(matches!(result, Err(SetGroupError::NotFound { id: 9999 })));
    }

    #[test]
    fn delete_planned_exercise_cascades_to_its_set_groups() {
        let mut conn = setup_test_db();
        let (planned_id, _a, _b, _c) = planned_exercise_with_three_groups(&mut conn);

        planned_exercises::delete(&conn, planned_id).expect("delete should succeed");

        let result = list(&conn, planned_id);
        assert!(matches!(
            result,
            Err(SetGroupError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn reorder_set_groups_rewrites_positions_in_the_given_order() {
        let mut conn = setup_test_db();
        let (planned_id, a, b, c) = planned_exercise_with_three_groups(&mut conn);

        reorder(&mut conn, planned_id, &[c.id(), a.id(), b.id()]).expect("reorder should succeed");

        let ordered = list(&conn, planned_id).expect("listing should succeed");
        let ids: Vec<i64> = ordered.iter().map(|g| g.id()).collect();
        assert_eq!(ids, vec![c.id(), a.id(), b.id()]);
        assert_eq!(ordered[0].position(), 0);
        assert_eq!(ordered[1].position(), 1);
        assert_eq!(ordered[2].position(), 2);
    }

    #[test]
    fn reorder_set_groups_returns_mismatch_when_ids_do_not_match_children() {
        let mut conn = setup_test_db();
        let (planned_id, a, _b, _c) = planned_exercise_with_three_groups(&mut conn);

        let result = reorder(&mut conn, planned_id, &[a.id()]);

        assert!(matches!(result, Err(SetGroupError::ReorderMismatch { .. })));
    }
}
