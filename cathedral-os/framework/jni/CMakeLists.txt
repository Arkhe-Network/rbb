# src/main/cpp/CMakeLists.txt
cmake_minimum_required(VERSION 3.18)
project("cathedral")

# Adiciona a biblioteca Rust (pré-compilada)
add_library(cathedral SHARED IMPORTED)
set_target_properties(cathedral PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libcathedral.so
)

# Adiciona o código JNI
add_library(cathedral_jni SHARED
    cathedral_jni.cpp
)

target_link_libraries(cathedral_jni
    cathedral
    log
)

# Copia a biblioteca para o APK
find_library(log-lib log)
target_link_libraries(cathedral_jni ${log-lib})
