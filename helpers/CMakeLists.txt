# Define project
cmake_minimum_required(VERSION 3.13)
project(helpers C)
set(CMAKE_C_STANDARD 11)


# Check for functions
function(check_function FN_NAME)
    include(CheckFunctionExists)
    check_function_exists(${FN_NAME} HAS_${FN_NAME})

    if(${HAS_${FN_NAME}})
        string(TOUPPER ${FN_NAME} FN_NAME)
        add_compile_definitions(HAS_${FN_NAME})
    endif()
endfunction()


# Check for safe memset-alternatives
check_function(memset_s)
check_function(SecureZeroMemory)
check_function(explicit_bzero)
check_function(explicit_memset)


# Define and install library
add_library(helpers STATIC helpers.c)
install(TARGETS helpers DESTINATION lib/)