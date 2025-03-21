# Use GoogleTest for testing
include(FetchContent)
FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG release-1.12.1
)
FetchContent_MakeAvailable(googletest)

file(GLOB_RECURSE TEST_SOURCES "*.cc")
#file(GLOB_RECURSE TEST_SOURCES "*.h")
# Test executable
add_executable(unit_tests ${TEST_SOURCES})
target_link_libraries(unit_tests PRIVATE
        ${PROJECT_NAME}  # Link to our static library
        gtest_main
)

# Register test
include(GoogleTest)
gtest_discover_tests(unit_tests)