//
// Created by Gavin Mead on 3/15/25.
//

#ifndef TRAINER_EXERCISE_H
#define TRAINER_EXERCISE_H

#include <optional>
#include <string>
#include <utility>

namespace trainer {
    namespace api {

        enum ExerciseType {
            BARBELL,
            KETTLEBELL
        };

        class Exercise {

        public:
            Exercise(std::string name, ExerciseType exerciseType) : Exercise(std::move(name), exerciseType, "") {}
            Exercise(std::string name, ExerciseType exerciseType, std::string description)
                : Exercise(0, name, exerciseType, std::move(description)) {};
            Exercise(int id, std::string name, ExerciseType exerciseType, std::string description)
                : id(id), name(std::move(name)), exerciseType(exerciseType), description(std::move(description)) {};
            ~Exercise() = default;

            std::optional<int> getId() {
                if (id != 0) {
                    return id;
                } else {
                    return std::nullopt;
                }
            }

            std::string getName() {
                return name;
            }

            std::string getDescription() {
                return description;
            }

            ExerciseType getExerciseType() {
                return exerciseType;
            }

        private:
            int id;
            std::string name;
            std::string description;
            ExerciseType exerciseType;
        };
    };
};


#endif//TRAINER_EXERCISE_H
