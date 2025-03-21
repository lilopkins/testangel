#include "testangel.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

uint64_t _dynamic_plugin_signature(void) {
    return 0;
}

/**
 * Return a list of instructions this engine supports
 */
ta_result * ta_request_instructions(
    ta_engine_metadata * pOutputEngineMetadata,
    ta_instruction_metadata *** parpOutputInstructions
) {
    pOutputEngineMetadata->iSupportsIpcVersion = 3;
    pOutputEngineMetadata->szFriendlyName = "Demo C Engine";
    pOutputEngineMetadata->szLuaName = "DemoC";
    pOutputEngineMetadata->szVersion = "0.0.0";
    pOutputEngineMetadata->szDescription = "An example of an engine implemented in C";

    ta_instruction_metadata *pInstructionMetadata = (ta_instruction_metadata *)malloc(sizeof(ta_instruction_metadata));
    pInstructionMetadata->szId = "demo-add";
    pInstructionMetadata->szLuaName = "Add";
    pInstructionMetadata->szFriendlyName = "Add";
    pInstructionMetadata->szDescription = "Add together two numbers";

    ta_instruction_named_kind *pParamA = (ta_instruction_named_kind *)malloc(sizeof(ta_instruction_named_kind));
    pParamA->szId = "a";
    pParamA->szName = "A";
    pParamA->kind = TA_PARAMETER_INTEGER;

    ta_instruction_named_kind *pParamB = (ta_instruction_named_kind *)malloc(sizeof(ta_instruction_named_kind));
    pParamB->szId = "b";
    pParamB->szName = "B";
    pParamB->kind = TA_PARAMETER_INTEGER;

    ta_instruction_named_kind **arpParameterList = (ta_instruction_named_kind **)malloc(3 * sizeof(ta_instruction_named_kind*));
    arpParameterList[0] = pParamA;
    arpParameterList[1] = pParamB;
    arpParameterList[2] = NULL;
    pInstructionMetadata->arpParameterList = arpParameterList;

    ta_instruction_named_kind *pOutputResult = (ta_instruction_named_kind *)malloc(sizeof(ta_instruction_named_kind));
    pOutputResult->szId = "result";
    pOutputResult->szName = "Result";
    pOutputResult->kind = TA_PARAMETER_INTEGER;

    ta_instruction_named_kind **arpOutputList = (ta_instruction_named_kind **)malloc(2 * sizeof(ta_instruction_named_kind*));
    arpOutputList[0] = pOutputResult;
    arpOutputList[1] = NULL;
    pInstructionMetadata->arpOutputList = arpOutputList;

    ta_instruction_metadata **ppInstructions = (ta_instruction_metadata **)malloc(2 * sizeof(ta_instruction_metadata*));
    ppInstructions[0] = pInstructionMetadata;
    ppInstructions[1] = NULL;
    (*parpOutputInstructions) = ppInstructions;

    ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
    pResult->code = TESTANGEL_OK;
    pResult->szReason = NULL;
    return pResult;
}

/**
* Execute an instruction
*/
ta_result * ta_execute(
    const char *szInstructionId,
    const ta_named_value *const *arpParameterList,
    uint32_t nParameterCount,
    ta_named_value ***parpOutputList,
    ta_evidence ***parpOutputEvidenceList
) {
    if (strcmp("demo-add", szInstructionId) != 0) {
        ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
        pResult->code = TESTANGEL_ERROR_INVALID_INSTRUCTION;
        pResult->szReason = "This engine only supports `demo-add`.";
        return pResult;
    }

    // Extract parameters A and B
    int32_t paramA = 0;
    int32_t paramB = 0;
    bool paramASupplied = false;
    bool paramBSupplied = false;

    for (uint32_t i = 0; i < nParameterCount; i++) {
        const ta_named_value * pParam = arpParameterList[i];
        if (strcmp(pParam->szName, "a") == 0) {
            paramASupplied = true;
            if (pParam->value.kind != TA_PARAMETER_INTEGER) {
                ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
                pResult->code = TESTANGEL_ERROR_INVALID_PARAMETER_TYPE;
                pResult->szReason = "Parameter A must be an integer!";
                return pResult;
            }
            paramA = *pParam->value.value.iValue;
        } else if (strcmp(pParam->szName, "b") == 0) {
            paramBSupplied = true;
            if (pParam->value.kind != TA_PARAMETER_INTEGER) {
                ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
                pResult->code = TESTANGEL_ERROR_INVALID_PARAMETER_TYPE;
                pResult->szReason = "Parameter B must be an integer!";
                return pResult;
            }
            paramB = *pParam->value.value.iValue;
        } else {
            ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
            pResult->code = TESTANGEL_ERROR_INVALID_PARAMETER;
            pResult->szReason = "One of the supplied parameters was unexpected!";
            return pResult;
        }
    }

    if (!paramASupplied) {
        ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
        pResult->code = TESTANGEL_ERROR_MISSING_PARAMETER;
        pResult->szReason = "Parameter `a` was not supplied";
        return pResult;
    }
    if (!paramBSupplied) {
        ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
        pResult->code = TESTANGEL_ERROR_MISSING_PARAMETER;
        pResult->szReason = "Parameter `b` was not supplied";
        return pResult;
    }

    printf("paramA = %d\n", paramA);
    printf("paramB = %d\n", paramB);

    int32_t *pOutputResult = (int32_t *)malloc(sizeof(int32_t));
    *pOutputResult = paramA + paramB;

    // Add evidence
    ta_evidence *pEvidence = (ta_evidence *)malloc(sizeof(ta_evidence));
    pEvidence->szLabel = "Sum";
    pEvidence->kind = TA_EVIDENCE_TEXTUAL;
    char *buf = (char *)malloc(255 * sizeof(char));
    sprintf(buf, "%d + %d = %d", paramA, paramB, *pOutputResult);
    buf = realloc(buf, (strlen(buf) + 1) * sizeof(char));
    pEvidence->value = buf;

    ta_evidence **arpEvidenceList = (ta_evidence **)malloc(2 * sizeof(ta_evidence*));
    arpEvidenceList[0] = pEvidence;
    arpEvidenceList[1] = NULL;
    (*parpOutputEvidenceList) = arpEvidenceList;

    // Set output
    ta_named_value *pOutput = (ta_named_value *)malloc(sizeof(ta_named_value));
    pOutput->szName = "result";
    pOutput->value.kind = TA_PARAMETER_INTEGER;
    pOutput->value.value.iValue = pOutputResult;

    ta_named_value **arpOutputList = (ta_named_value **)malloc(2 * sizeof(ta_named_value*));
    arpOutputList[0] = pOutput;
    arpOutputList[1] = NULL;
    (*parpOutputList) = arpOutputList;

    ta_result *pResult = (ta_result *)malloc(sizeof(ta_result));
    pResult->code = TESTANGEL_OK;
    pResult->szReason = NULL;
    return pResult;
}

/**
* Reset engine state
*/
ta_result * ta_reset_state(void) {
    ta_result * pResult = (ta_result *) malloc(sizeof(ta_result));
    pResult->code = TESTANGEL_OK;
    pResult->szReason = NULL;
    return pResult;
}

/**
* Free a result struct
*/
void ta_free_result(const ta_result *pTarget) {
    if (pTarget->szReason != NULL) {
        free((void *)pTarget->szReason);
    }
    free((void *)pTarget);
}

/**
* Free an engine metadata struct
*/
void ta_free_engine_metadata(const ta_engine_metadata *pTarget) {
    // Nothing to do in this implementation, all the metadata is static.
}

/**
* Free an array of instruction metadata structs
*/
void ta_free_instruction_metadata_array(const ta_instruction_metadata *const *arpTarget) {
    // Loop through array and free each instruction
    for (uint32_t i = 0; arpTarget[i] != NULL; i++) {
        const ta_instruction_metadata *const pMeta = arpTarget[i];
        for (uint32_t j = 0; pMeta->arpParameterList[j] != NULL; j++) {
            ta_instruction_named_kind *pInk = pMeta->arpParameterList[j];
            free((void *)pInk);
        }
        for (uint32_t j = 0; pMeta->arpOutputList[j] != NULL; j++) {
            ta_instruction_named_kind *pInk = pMeta->arpOutputList[j];
            free((void *)pInk);
        }
        free((void *)pMeta);
    }
    free((void *)arpTarget);
}

/**
* Free an array of named value structs
*/
void ta_free_named_value_array(const ta_named_value *const *arpTarget) {
    for (uint32_t i = 0; arpTarget[i] != NULL; i++) {
        const ta_named_value *const pNamedValue = arpTarget[i];
        free((void *)pNamedValue->szName);
        // Type here doesn't matter as it'll be a void* either way
        free((void *)pNamedValue->value.value.iValue);
        free((void *)pNamedValue);
    }
    free((void *)arpTarget);
}

/**
* Free an array of evidence structs
*/
void ta_free_evidence_array(const ta_evidence *const *arpTarget) {
    for (uint32_t i = 0; arpTarget[i] != NULL; i++) {
        const ta_evidence *const pEvidence = arpTarget[i];
        free((void *)pEvidence->szLabel);
        free((void *)pEvidence->value);
        free((void *)pEvidence);
    }
    free((void *)arpTarget);
}
