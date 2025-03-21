function(trainer_install_library_headers)
    # Find any headers and install them relative to the source tree in include.
    file(GLOB _hdrs "*.h")
    if(NOT "${_hdrs}" STREQUAL "")
        cmake_path(
                RELATIVE_PATH
                CMAKE_CURRENT_SOURCE_DIR
                BASE_DIRECTORY
                "${CMAKE_SOURCE_DIR}"
                OUTPUT_VARIABLE
                _hdr_dir)
        install(FILES ${_hdrs} DESTINATION include/${_hdr_dir})
    endif()
endfunction()

