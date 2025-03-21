include(FetchContent)

set(TRAINER_GTEST_VERSION 1.16.0)
set(TRAINER_GTEST_REPO "https://github.com/google/googletest/releases/download/v${TRAINER_GTEST_VERSION}/googletest-${TRAINER_GTEST_VERSION}.tar.gz")

FetchContent_Declare(
        googletest
        URL ${TRAINER_GTEST_REPO}
)
FetchContent_MakeAvailable(googletest)
