include(vcpkg_common_functions)

set(VERSION 1.8.65)
set(SOURCE_PATH ${CURRENT_BUILDTREES_DIR}/src/tdlib-${VERSION})

vcpkg_from_github(
        OUT_SOURCE_PATH SOURCE_PATH
        REPO tdlib/td
        REF 022d60202e446ad1287b9fb68e687c8a0760788b
        SHA512 7f6446c2c2937dba8971d8b13b67ae7a0056aa812a9ae55bcbdb7875213421262d09613be1869525b2e3e8c2f4b494b7521d0f36e7257e87f5d0d0fa867f604c
        HEAD_REF master
)

if (VCPKG_LIBRARY_LINKAGE STREQUAL static)
    vcpkg_apply_patches(
            SOURCE_PATH ${SOURCE_PATH}
            PATCHES
            ${CMAKE_CURRENT_LIST_DIR}/static.patch
            ${CMAKE_CURRENT_LIST_DIR}/openssl.patch
    )
endif()

vcpkg_find_acquire_program(GPERF)
vcpkg_configure_cmake(
        SOURCE_PATH ${SOURCE_PATH}
        PREFER_NINJA
        OPTIONS -DGPERF_EXECUTABLE:FILEPATH="${GPERF}"
)

vcpkg_install_cmake()
vcpkg_copy_pdbs()
vcpkg_fixup_cmake_targets(CONFIG_PATH lib/cmake/Td)

configure_file(${SOURCE_PATH}/LICENSE_1_0.txt ${CURRENT_PACKAGES_DIR}/share/${PORT}/copyright COPYONLY)

file(REMOVE_RECURSE ${CURRENT_PACKAGES_DIR}/debug/include)
if (VCPKG_LIBRARY_LINKAGE STREQUAL static)
    file(REMOVE_RECURSE
            ${CURRENT_PACKAGES_DIR}/bin
            ${CURRENT_PACKAGES_DIR}/debug/bin)
endif()