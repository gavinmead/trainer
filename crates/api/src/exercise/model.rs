#[derive(Clone, Debug, PartialEq, Copy)]
#[non_exhaustive]
pub enum ExerciseType {
    Barbell,
    KettleBell,
    BodyWeight,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] //this is temporary as code base evolves
pub struct Exercise {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub exercise_type: ExerciseType,
}

impl From<ExerciseType> for i64 {
    fn from(value: ExerciseType) -> Self {
        match value {
            ExerciseType::Barbell => 0,
            ExerciseType::KettleBell => 1,
            ExerciseType::BodyWeight => 2,
        }
    }
}

impl From<i64> for ExerciseType {
    fn from(value: i64) -> Self {
        match value {
            0 => ExerciseType::Barbell,
            1 => ExerciseType::KettleBell,
            2 => ExerciseType::BodyWeight,
            _ => panic!("unsupported value"),
        }
    }
}

impl From<String> for ExerciseType {
    fn from(value: String) -> Self {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "barbell" => ExerciseType::Barbell,
            "bb" => ExerciseType::Barbell,
            "kettlebell" => ExerciseType::KettleBell,
            "kb" => ExerciseType::KettleBell,
            "bw" => ExerciseType::BodyWeight,
            "bodyweight" => ExerciseType::BodyWeight,
            _ => panic!("unsupported value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_to_exercise_type_ok() {
        let bbs = vec![
            "Barbell".to_string(),
            "BARBELL".to_string(),
            "bArBeLl".to_string(),
            "bb".to_string(),
            "BB".to_string(),
            "bB".to_string(),
        ];
        let kbs = vec![
            "Kettlebell".to_string(),
            "KETTLEBELL".to_string(),
            "kEtTlEbElL".to_string(),
            "kb".to_string(),
            "KB".to_string(),
            "kB".to_string(),
        ];

        let bws = vec![
            "BW".to_string(),
            "bw".to_string(),
            "BodyWeight".to_string(),
            "bOdYwEiGhT".to_string(),
            "Bw".to_string(),
            "bW".to_string(),
        ];

        for bb in bbs {
            let et: ExerciseType = bb.into();
            assert_eq!(et, ExerciseType::Barbell)
        }

        for kb in kbs {
            let et: ExerciseType = kb.into();
            assert_eq!(et, ExerciseType::KettleBell)
        }

        for bw in bws {
            let eb: ExerciseType = bw.into();
            assert_eq!(eb, ExerciseType::BodyWeight)
        }
    }

    #[test]
    #[should_panic]
    fn from_string_to_exercise_type_fail() {
        let _: ExerciseType = "not_found".to_string().into();
    }

    #[test]
    #[should_panic]
    fn from_invalid_string_to_exercise_type_fail() {
        let _: ExerciseType = 1000.into();
    }
}
