cc_test(
    name = "my_lib_test",
    srcs = ["my_lib_test.cc"],
    deps = [
        "//src/lib:my_lib",
        "@googletest//:gtest_main",
    ],
)

test_suite(
    name = "all_tests",
    tests = [
        ":my_lib_test",
    ],
)
