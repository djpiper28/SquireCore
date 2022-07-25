if(WIN32)
  set(EXT "lib")
elseif("${CMAKE_SYSTEM_NAME}" STREQUAL "Macos")
  set(EXT "dylib")
else()
  set(EXT "a")
endif()

set(RUST_PART_CXX_FILE "${CMAKE_CURRENT_BINARY_DIR}/squire_core.${EXT}")
set(RUST_PART_H_FILE "${CMAKE_CURRENT_BINARY_DIR}/squire_core.h")
message(
  STATUS
    "Building rust with options: ${RUST_FLAGS}, linking ${RUST_PART_CXX_FILE}")
if(WIN32)
  add_custom_command(
    OUTPUT ${RUST_PART_H_FILE}
    COMMAND ${CMAKE_COMMAND} -E env RUSTFLAGS="${RUST_FLAGS}" "ffi_build.bat"
    COMMAND
      ${CMAKE_COMMAND} -E copy
      "${CMAKE_CURRENT_SOURCE_DIR}/target/debug/squire_lib.${EXT}"
      "${RUST_PART_CXX_FILE}"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/squire_core.h"
            "${RUST_PART_H_FILE}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
else()
  add_custom_command(
    OUTPUT ${RUST_PART_H_FILE}
    COMMAND ${CMAKE_COMMAND} -E env RUSTFLAGS="${RUST_FLAGS}" bash
            "ffi_build.sh"
    COMMAND
      ${CMAKE_COMMAND} -E copy
      "${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libsquire_lib.${EXT}"
      "${RUST_PART_CXX_FILE}"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/squire_core.h"
            "${RUST_PART_H_FILE}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
endif()
add_custom_target(RUST_PART_H DEPENDS ${RUST_PART_H_FILE})

add_library(squire_core)
set_target_properties(squire_core PROPERTIES IMPORTED_LOCATION
                                             ${RUST_PART_CXX_FILE})
target_link_libraries(squire_core INTERFACE ${RUST_PART_CXX_FILE})
add_dependencies(squire_core RUST_PART_H)

add_test(
  NAME squire_core_test
  COMMAND cargo test
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

message(STATUS "Squire Core FFI is being used")
