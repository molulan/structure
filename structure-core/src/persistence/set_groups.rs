use crate::domain::planning::{
    ExerciseType, Intensity, PercentOneRepMax, PrescribedSetType, RepTarget, Rir, Rpe, SetGroup,
    SetGroupType, SetGroupValidationError, Weight, WeightUnit,
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
    #[error("corrupt set group data in the database: {0}")]
    Corrupt(String),
}

fn corrupt(detail: impl std::fmt::Display) -> SetGroupError {
    SetGroupError::Corrupt(detail.to_string())
}

pub(super) fn create_set_groups_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS set_groups (
            id INTEGER PRIMARY KEY,
            planned_exercise_id INTEGER NOT NULL REFERENCES planned_exercises(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            set_type TEXT NOT NULL CHECK(
                set_type IN ('Regular', 'Myorep', 'MyorepMatch', 'Drop')
            ),
            number_of_sets INTEGER NOT NULL CHECK(number_of_sets > 0),
            rep_min INTEGER CHECK(rep_min IS NULL OR rep_min > 0),
            -- A non-NULL rep_max requires a non-NULL rep_min (and must ascend), so
            -- a MyorepMatch row (rep_min NULL) cannot carry a stray rep_max.
            rep_max INTEGER CHECK(rep_max IS NULL OR (rep_min IS NOT NULL AND rep_max > rep_min)),
            intensity_type TEXT CHECK(
                intensity_type IN ('Rir', 'Rpe', 'PercentOneRepMax', 'TargetWeight', 'WeightIncrement')
            ),
            intensity_value REAL,
            intensity_weight_unit TEXT CHECK(intensity_weight_unit IN ('Kg', 'Lbs')),
            UNIQUE(planned_exercise_id, position),
            -- A MyorepMatch group carries no prescription; every other set type must.
            CHECK((set_type = 'MyorepMatch') = (rep_min IS NULL)),
            CHECK((set_type = 'MyorepMatch') = (intensity_type IS NULL)),
            CHECK((intensity_type IS NULL) = (intensity_value IS NULL)),
            -- COALESCE pins the weight unit NULL when intensity_type is NULL
            -- (MyorepMatch); a bare `intensity_type IN (...)` would be NULL there
            -- and pass the check vacuously.
            CHECK(
                (intensity_weight_unit IS NOT NULL)
                    = COALESCE(intensity_type IN ('TargetWeight', 'WeightIncrement'), 0)
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
    number_of_sets: u32,
    set_group_type: SetGroupType,
) -> Result<SetGroup, SetGroupError> {
    // Surface the typed ZeroSets error here: it's the one domain invariant that
    // overlaps a DB CHECK, so without this guard a zero count would fail at the
    // INSERT and surface as an opaque Database error instead.
    if number_of_sets == 0 {
        return Err(SetGroupValidationError::ZeroSets.into());
    }

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

    let columns = encode_set_group_type(set_group_type);

    tx.execute(
        "INSERT INTO set_groups
            (planned_exercise_id, position, set_type, number_of_sets,
             rep_min, rep_max, intensity_type, intensity_value, intensity_weight_unit)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            planned_exercise_id,
            position,
            columns.set_type,
            number_of_sets,
            columns.rep_min,
            columns.rep_max,
            columns.intensity_type,
            columns.intensity_value,
            columns.intensity_weight_unit,
        ],
    )?;

    let id = tx.last_insert_rowid();
    let set_group = SetGroup::new(id, position, exercise_type, number_of_sets, set_group_type)?;
    tx.commit()?;
    Ok(set_group)
}

pub fn update(
    conn: &mut Connection,
    id: i64,
    number_of_sets: u32,
    set_group_type: SetGroupType,
) -> Result<SetGroup, SetGroupError> {
    if number_of_sets == 0 {
        return Err(SetGroupValidationError::ZeroSets.into());
    }

    let tx = conn.transaction()?;

    let Some(exercise_type) = set_group_exercise_type(&tx, id)? else {
        return Err(SetGroupError::NotFound { id });
    };

    let columns = encode_set_group_type(set_group_type);

    let position: Option<i64> = tx
        .query_row(
            "UPDATE set_groups SET
                set_type = ?1, number_of_sets = ?2, rep_min = ?3, rep_max = ?4,
                intensity_type = ?5, intensity_value = ?6, intensity_weight_unit = ?7
             WHERE id = ?8
             RETURNING position",
            params![
                columns.set_type,
                number_of_sets,
                columns.rep_min,
                columns.rep_max,
                columns.intensity_type,
                columns.intensity_value,
                columns.intensity_weight_unit,
                id,
            ],
            |row| row.get(0),
        )
        .optional()?;

    let position = match position {
        Some(position) => decode_u32(position, "position")?,
        None => return Err(SetGroupError::NotFound { id }),
    };

    let set_group = SetGroup::new(id, position, exercise_type, number_of_sets, set_group_type)?;
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

fn decode_row(row: &rusqlite::Row<'_>) -> Result<SetGroup, SetGroupError> {
    let id: i64 = row.get(0)?;
    let position = decode_u32(row.get(1)?, "position")?;
    let set_type: String = row.get(2)?;
    let number_of_sets = decode_u32(row.get(3)?, "number_of_sets")?;
    let rep_min: Option<i64> = row.get(4)?;
    let rep_max: Option<i64> = row.get(5)?;
    let intensity_type: Option<String> = row.get(6)?;
    let intensity_value: Option<f64> = row.get(7)?;
    let intensity_weight_unit: Option<String> = row.get(8)?;

    let set_group_type = decode_set_group_type(
        &set_type,
        rep_min,
        rep_max,
        intensity_type.as_deref(),
        intensity_value,
        intensity_weight_unit.as_deref(),
    )?;

    Ok(SetGroup::new_unchecked(
        id,
        position,
        number_of_sets,
        set_group_type,
    ))
}

pub fn list(conn: &Connection, planned_exercise_id: i64) -> Result<Vec<SetGroup>, SetGroupError> {
    if planned_exercise_type(conn, planned_exercise_id)?.is_none() {
        return Err(SetGroupError::AssociatedPlannedExerciseNotFound {
            id: planned_exercise_id,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT id, position, set_type, number_of_sets, rep_min, rep_max,
                intensity_type, intensity_value, intensity_weight_unit
         FROM set_groups
         WHERE planned_exercise_id = ?1
         ORDER BY position ASC",
    )?;

    // `query_map`'s closure can only yield `rusqlite::Result`, so we iterate
    // manually to let a decode error surface as a typed `SetGroupError`.
    let mut rows = stmt.query([planned_exercise_id])?;
    let mut groups = Vec::new();
    while let Some(row) = rows.next()? {
        groups.push(decode_row(row)?);
    }
    Ok(groups)
}

/// The persisted column values for a set group's [`SetGroupType`]. A
/// `MyorepMatch` leaves every prescription column `NULL`; a `Prescribed` group
/// fills them from its set type, reps, and intensity.
struct SetGroupTypeColumns {
    set_type: &'static str,
    rep_min: Option<i64>,
    rep_max: Option<i64>,
    intensity_type: Option<&'static str>,
    intensity_value: Option<f64>,
    intensity_weight_unit: Option<&'static str>,
}

fn encode_set_group_type(set_group_type: SetGroupType) -> SetGroupTypeColumns {
    match set_group_type {
        SetGroupType::MyorepMatch => SetGroupTypeColumns {
            set_type: "MyorepMatch",
            rep_min: None,
            rep_max: None,
            intensity_type: None,
            intensity_value: None,
            intensity_weight_unit: None,
        },
        SetGroupType::Prescribed {
            set_type,
            reps,
            intensity,
        } => {
            let (rep_min, rep_max) = encode_reps(reps);
            let (intensity_type, intensity_value, intensity_weight_unit) =
                encode_intensity(intensity);
            SetGroupTypeColumns {
                set_type: set_type.as_str(),
                rep_min: Some(rep_min),
                rep_max,
                intensity_type: Some(intensity_type),
                intensity_value: Some(intensity_value),
                intensity_weight_unit,
            }
        }
    }
}

fn decode_set_group_type(
    set_type: &str,
    rep_min: Option<i64>,
    rep_max: Option<i64>,
    intensity_type: Option<&str>,
    intensity_value: Option<f64>,
    intensity_weight_unit: Option<&str>,
) -> Result<SetGroupType, SetGroupError> {
    if set_type == "MyorepMatch" {
        return Ok(SetGroupType::MyorepMatch);
    }

    let rep_min = rep_min.ok_or_else(|| corrupt("prescribed set group has NULL rep_min"))?;
    let reps = decode_reps(rep_min, rep_max)?;
    let intensity_type =
        intensity_type.ok_or_else(|| corrupt("prescribed set group has NULL intensity_type"))?;
    let intensity_value =
        intensity_value.ok_or_else(|| corrupt("prescribed set group has NULL intensity_value"))?;
    let intensity = decode_intensity(intensity_type, intensity_value, intensity_weight_unit)?;

    Ok(SetGroupType::Prescribed {
        set_type: prescribed_set_type_from_str(set_type)?,
        reps,
        intensity,
    })
}

/// Reads an `i64` column that must fit a `u32` (positions and counts).
fn decode_u32(value: i64, column: &str) -> Result<u32, SetGroupError> {
    u32::try_from(value).map_err(|_| corrupt(format!("{column} {value} is out of u32 range")))
}

/// Reads a `REAL` column that must hold a whole number: the integer intensities
/// (`Rir`/`Rpe`/`PercentOneRepMax`) share the `intensity_value` column with
/// genuine float weights, so a fractional value there is corruption.
fn decode_whole(value: f64, kind: &str) -> Result<i64, SetGroupError> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(corrupt(format!(
            "{kind} value {value} is not a whole number"
        )));
    }
    Ok(value as i64)
}

fn encode_reps(reps: RepTarget) -> (i64, Option<i64>) {
    match reps {
        RepTarget::Exact(count) => (count.value() as i64, None),
        RepTarget::Range(range) => (range.min() as i64, Some(range.max() as i64)),
    }
}

fn decode_reps(rep_min: i64, rep_max: Option<i64>) -> Result<RepTarget, SetGroupError> {
    let min = decode_u32(rep_min, "rep_min")?;
    match rep_max {
        None => RepTarget::exact(min).map_err(corrupt),
        Some(max) => {
            let max = decode_u32(max, "rep_max")?;
            RepTarget::range(min, max).map_err(corrupt)
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

fn decode_intensity(
    intensity_type: &str,
    value: f64,
    unit: Option<&str>,
) -> Result<Intensity, SetGroupError> {
    let intensity = match intensity_type {
        "Rir" => {
            let value = i8::try_from(decode_whole(value, "Rir")?).map_err(corrupt)?;
            Intensity::Rir(Rir::new(value).map_err(corrupt)?)
        }
        "Rpe" => {
            let value = u8::try_from(decode_whole(value, "Rpe")?).map_err(corrupt)?;
            Intensity::Rpe(Rpe::new(value).map_err(corrupt)?)
        }
        "PercentOneRepMax" => {
            let value = u8::try_from(decode_whole(value, "PercentOneRepMax")?).map_err(corrupt)?;
            Intensity::PercentOneRepMax(PercentOneRepMax::new(value).map_err(corrupt)?)
        }
        "TargetWeight" => Intensity::TargetWeight(decode_weight(value, unit)?),
        "WeightIncrement" => Intensity::WeightIncrement(decode_weight(value, unit)?),
        other => return Err(corrupt(format!("unknown intensity_type: {other}"))),
    };
    Ok(intensity)
}

fn decode_weight(value: f64, unit: Option<&str>) -> Result<Weight, SetGroupError> {
    let unit = unit.ok_or_else(|| corrupt("weight intensity has NULL weight unit"))?;
    Ok(Weight::new(value, weight_unit_from_str(unit)?))
}

fn prescribed_set_type_from_str(s: &str) -> Result<PrescribedSetType, SetGroupError> {
    match s {
        "Regular" => Ok(PrescribedSetType::Regular),
        "Myorep" => Ok(PrescribedSetType::Myorep),
        "Drop" => Ok(PrescribedSetType::Drop),
        other => Err(corrupt(format!("unknown prescribed set_type: {other}"))),
    }
}

fn weight_unit_to_str(unit: WeightUnit) -> &'static str {
    match unit {
        WeightUnit::Kg => "Kg",
        WeightUnit::Lbs => "Lbs",
    }
}

fn weight_unit_from_str(s: &str) -> Result<WeightUnit, SetGroupError> {
    match s {
        "Kg" => Ok(WeightUnit::Kg),
        "Lbs" => Ok(WeightUnit::Lbs),
        other => Err(corrupt(format!("unknown weight_unit: {other}"))),
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

    fn regular(reps: RepTarget, intensity: Intensity) -> SetGroupType {
        SetGroupType::Prescribed {
            set_type: PrescribedSetType::Regular,
            reps,
            intensity,
        }
    }

    fn regular_rir() -> SetGroupType {
        regular(
            RepTarget::exact(5).unwrap(),
            Intensity::Rir(Rir::new(2).unwrap()),
        )
    }

    #[test]
    fn create_set_group_on_existing_planned_exercise_succeeds() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = create(
            &mut conn,
            planned.id(),
            3,
            regular(
                RepTarget::range(8, 12).unwrap(),
                Intensity::Rir(Rir::new(2).unwrap()),
            ),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn create_set_group_for_nonexistent_planned_exercise_returns_error() {
        let mut conn = setup_test_db();

        let result = create(&mut conn, 9999, 3, regular_rir());

        assert!(matches!(
            result,
            Err(SetGroupError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn create_set_group_with_zero_sets_returns_invalid_not_an_opaque_db_error() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = create(&mut conn, planned.id(), 0, regular_rir());

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(SetGroupValidationError::ZeroSets))
        ));
    }

    #[test]
    fn update_set_group_with_zero_sets_returns_invalid() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        let group = create(&mut conn, planned.id(), 3, regular_rir())
            .expect("set group creation should succeed");

        let result = update(&mut conn, group.id(), 0, regular_rir());

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(SetGroupValidationError::ZeroSets))
        ));
    }

    #[test]
    fn schema_rejects_a_myorep_match_row_with_a_stray_rep_max() {
        let conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = conn.execute(
            "INSERT INTO set_groups
                (planned_exercise_id, position, set_type, number_of_sets, rep_max)
             VALUES (?1, 0, 'MyorepMatch', 3, 5)",
            [planned.id()],
        );

        assert!(
            result.is_err(),
            "a MyorepMatch row must not carry a rep_max"
        );
    }

    #[test]
    fn schema_rejects_a_myorep_match_row_with_a_stray_weight_unit() {
        let conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = conn.execute(
            "INSERT INTO set_groups
                (planned_exercise_id, position, set_type, number_of_sets, intensity_weight_unit)
             VALUES (?1, 0, 'MyorepMatch', 3, 'Kg')",
            [planned.id()],
        );

        assert!(
            result.is_err(),
            "a MyorepMatch row must not carry a weight unit"
        );
    }

    #[test]
    fn list_surfaces_a_corrupt_row_as_a_typed_error_rather_than_panicking() {
        let conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        // A fractional Rir value passes every CHECK (intensity_value is REAL) but
        // is not a whole number, so decoding it must fail rather than truncate.
        conn.execute(
            "INSERT INTO set_groups
                (planned_exercise_id, position, set_type, number_of_sets,
                 rep_min, intensity_type, intensity_value)
             VALUES (?1, 0, 'Regular', 3, 5, 'Rir', 2.5)",
            [planned.id()],
        )
        .expect("the raw row itself satisfies the CHECK constraints");

        let result = list(&conn, planned.id());

        assert!(matches!(result, Err(SetGroupError::Corrupt(_))));
    }

    #[test]
    fn create_set_group_with_proximity_intensity_on_failure_set_type_returns_invalid() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let result = create(
            &mut conn,
            planned.id(),
            3,
            SetGroupType::Prescribed {
                set_type: PrescribedSetType::Myorep,
                reps: RepTarget::exact(10).unwrap(),
                intensity: Intensity::Rir(Rir::new(1).unwrap()),
            },
        );

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(
                SetGroupValidationError::IntensityIncompatibleWithSetType { .. }
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
            3,
            regular(
                RepTarget::exact(10).unwrap(),
                Intensity::TargetWeight(Weight::new(20.0, WeightUnit::Kg)),
            ),
        );

        assert!(matches!(
            result,
            Err(SetGroupError::Invalid(
                SetGroupValidationError::WeightIntensityOnBodyweight { .. }
            ))
        ));
    }

    #[test]
    fn create_myorep_match_set_group_without_a_prescription_succeeds() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let group = create(&mut conn, planned.id(), 3, SetGroupType::MyorepMatch)
            .expect("a myorep-match group needs no prescription");

        assert_eq!(group.set_group_type(), SetGroupType::MyorepMatch);
    }

    #[test]
    fn set_groups_get_sequential_positions() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        let new_group = |conn: &mut Connection| {
            create(conn, planned.id(), 3, regular_rir()).expect("set group creation should succeed")
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
    fn every_set_group_type_round_trips_through_the_database() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);

        let created: Vec<SetGroup> = [
            regular(
                RepTarget::exact(5).unwrap(),
                Intensity::Rir(Rir::new(0).unwrap()),
            ),
            regular(
                RepTarget::range(8, 12).unwrap(),
                Intensity::Rpe(Rpe::new(9).unwrap()),
            ),
            regular(
                RepTarget::exact(3).unwrap(),
                Intensity::PercentOneRepMax(PercentOneRepMax::new(85).unwrap()),
            ),
            SetGroupType::Prescribed {
                set_type: PrescribedSetType::Myorep,
                reps: RepTarget::range(6, 10).unwrap(),
                intensity: Intensity::TargetWeight(Weight::new(100.0, WeightUnit::Kg)),
            },
            SetGroupType::Prescribed {
                set_type: PrescribedSetType::Drop,
                reps: RepTarget::exact(8).unwrap(),
                intensity: Intensity::WeightIncrement(Weight::new(-2.5, WeightUnit::Lbs)),
            },
            SetGroupType::MyorepMatch,
        ]
        .into_iter()
        .map(|set_group_type| {
            create(&mut conn, planned.id(), 4, set_group_type)
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
        let group = create(&mut conn, planned.id(), 3, regular_rir())
            .expect("set group creation should succeed");

        let new_set_group_type = SetGroupType::Prescribed {
            set_type: PrescribedSetType::Myorep,
            reps: RepTarget::range(10, 20).unwrap(),
            intensity: Intensity::TargetWeight(Weight::new(40.0, WeightUnit::Kg)),
        };
        let updated =
            update(&mut conn, group.id(), 2, new_set_group_type).expect("update should succeed");

        assert_eq!(updated.number_of_sets(), 2);
        assert_eq!(updated.set_group_type(), new_set_group_type);
        assert_eq!(updated.position(), group.position());

        let listed = list(&conn, planned.id()).expect("listing should succeed");
        assert_eq!(listed, vec![updated]);
    }

    #[test]
    fn update_set_group_to_myorep_match_clears_its_prescription() {
        let mut conn = setup_test_db();
        let planned = weighted_planned_exercise(&conn);
        let group = create(&mut conn, planned.id(), 3, regular_rir())
            .expect("set group creation should succeed");

        let updated = update(&mut conn, group.id(), 3, SetGroupType::MyorepMatch)
            .expect("update should succeed");

        assert_eq!(updated.set_group_type(), SetGroupType::MyorepMatch);

        let listed = list(&conn, planned.id()).expect("listing should succeed");
        assert_eq!(listed, vec![updated]);
    }

    #[test]
    fn update_set_group_returns_not_found_when_it_does_not_exist() {
        let mut conn = setup_test_db();

        let result = update(&mut conn, 9999, 3, regular_rir());

        assert!(matches!(result, Err(SetGroupError::NotFound { id: 9999 })));
    }

    /// Returns the planned exercise id and three of its set groups.
    fn planned_exercise_with_three_groups(
        conn: &mut Connection,
    ) -> (i64, SetGroup, SetGroup, SetGroup) {
        let planned = weighted_planned_exercise(conn);
        let new_group = |conn: &mut Connection| {
            create(conn, planned.id(), 3, regular_rir()).expect("set group creation should succeed")
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

        let next = create(&mut conn, planned_id, 3, regular_rir())
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
