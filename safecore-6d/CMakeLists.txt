idf_component_register(
    SRCS "src/physics/partition.c"
         "src/physics/deflection_unwrap.c"
         "src/constitutional/c10_baryon_ratio.c"
         "src/constitutional/c11_parity_violation.c"
         "src/concept/golden_angle.c"
    INCLUDE_DIRS "include"
    REQUIRES esp_timer
)
