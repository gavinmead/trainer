cmake_minimum_required(VERSION 3.30)
project(trainer)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(CTest)

# Add clang-format target if the program is found
find_program(CLANG_FORMAT "clang-format")
if(CLANG_FORMAT)
    # Get all project files
    file(GLOB_RECURSE ALL_SOURCE_FILES
            ${PROJECT_SOURCE_DIR}/src/*.cc
            ${PROJECT_SOURCE_DIR}/include/**/*.h
            ${PROJECT_SOURCE_DIR}/include/*.h
    )

    add_custom_target(
            format
            COMMAND ${CLANG_FORMAT}
            -i
            ${ALL_SOURCE_FILES}
    )
else()
    message(STATUS "clang-format not found, format target not available")
endif()

# Coverage option
option(ENABLE_COVERAGE "Enable coverage reporting" OFF)
if(ENABLE_CODE_COVERAGE)
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        message(STATUS "Enabling code coverage with Clang")
        add_compile_options(-fprofile-instr-generate -fcoverage-mapping)
        add_link_options(-fprofile-instr-generate -fcoverage-mapping)

        # Add coverage targets
        add_custom_target(coverage
                COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/coverage
                COMMAND LLVM_PROFILE_FILE=${CMAKE_BINARY_DIR}/coverage/coverage.profraw ${CMAKE_CTEST_COMMAND} --output-on-failure
                COMMAND llvm-profdata merge -sparse ${CMAKE_BINARY_DIR}/coverage/coverage.profraw -o ${CMAKE_BINARY_DIR}/coverage/coverage.profdata
                COMMAND llvm-cov show $<TARGET_FILE:${PROJECT_NAME}_tests> -instr-profile=${CMAKE_BINARY_DIR}/coverage/coverage.profdata -show-line-counts-or-regions -output-dir=${CMAKE_BINARY_DIR}/coverage -format=html
                COMMAND llvm-cov export $<TARGET_FILE:${PROJECT_NAME}_tests> -instr-profile=${CMAKE_BINARY_DIR}/coverage/coverage.profdata -format=lcov > ${CMAKE_BINARY_DIR}/coverage/coverage.lcov
                WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                COMMENT "Generating coverage report"
                DEPENDS ${PROJECT_NAME}_tests
        )

        add_custom_target(codecov
                COMMAND codecov -f ${CMAKE_BINARY_DIR}/coverage/coverage.lcov
                WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                COMMENT "Uploading coverage report to codecov.io"
                DEPENDS coverage
        )
    else()
        message(WARNING "Code coverage is only supported with Clang compiler")
    endif()
endif()

file(GLOB_RECURSE LIBRARY_SOURCES "src/*.cc")

add_library(${PROJECT_NAME} STATIC ${LIBRARY_SOURCES})
target_include_directories(${PROJECT_NAME}
        PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)


# Testing setup
option(BUILD_TESTS "Build tests" ON)
if(BUILD_TESTS)
    enable_testing()
    add_subdirectory(test)
endif()

