//
// Created by Gavin Mead on 3/15/25.
//
#include "trainer/api/exercise.h"
#include <gtest/gtest.h>
#include <optional>

using namespace trainer::api;

TEST(ExerciseTest, ExerciseConstructor) {
    auto exercise = Exercise("Squat", BARBELL);
    ASSERT_EQ(exercise.getName(), "Squat");
    ASSERT_EQ(exercise.getExerciseType(), BARBELL);
    ASSERT_EQ(exercise.getDescription(), "");
}

TEST(ExerciseTest, EmptyId) {
    auto exercise = Exercise("Squat", BARBELL);
    ASSERT_EQ(exercise.getId(), std::nullopt);
}

