#pragma once
#include <stddef.h>
typedef struct {
    float r;
    float phi;
    float psi;
    int level;
    int parent;
} ConceptV2;
typedef struct {
    ConceptV2 *concepts;
    size_t count;
    float aq;
    int depth;
    int branching;
} ConceptSpaceV2;
ConceptSpaceV2* concept_space_v2_new(float aq, int depth, int branching);
void concept_space_v2_free(ConceptSpaceV2 *cs);
