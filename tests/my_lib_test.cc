// tests/my_lib_test.cpp
#include "src/lib/my_lib.h"
#include <gtest/gtest.h>

// Test the get_answer function
TEST(MyLibTest, GetAnswerReturns42) {
    EXPECT_EQ(my_lib::get_answer(), 42);
}

// Test the calculate template function
TEST(MyLibTest, CalculateTemplate) {
    EXPECT_EQ(my_lib::calculate<42>(), 42);
}

