#include <jni.h>
#include <string>
#include <vector>
#include "cathedral_core.h"  // Gerado por cbindgen

// ============================================================================
// Auxiliares
// ============================================================================

inline void throwCathedralException(JNIEnv* env, CathedralError err) {
    jclass clazz = env->FindClass("cathedral/CathedralException");
    if (clazz != nullptr) {
        env->ThrowNew(clazz, cathedral_error_to_string(err));
    }
}

inline jlong toJavaHandle(void* ptr) {
    return reinterpret_cast<jlong>(ptr);
}

inline void* toNativeHandle(jlong handle) {
    return reinterpret_cast<void*>(handle);
}

// ============================================================================
// Agent
// ============================================================================

extern "C" {

JNIEXPORT jlong JNICALL
Java_cathedral_Agent_nativeCreateAgent(JNIEnv* env, jclass clazz,
                                       jstring identity, jbyteArray pubKey, jbyteArray privKey) {
    const char* id = env->GetStringUTFChars(identity, nullptr);
    jsize pubLen = env->GetArrayLength(pubKey);
    jbyte* pub = env->GetByteArrayElements(pubKey, nullptr);
    jsize privLen = env->GetArrayLength(privKey);
    jbyte* priv = env->GetByteArrayElements(privKey, nullptr);

    CathedralAgentHandle handle = cathedral_agent_create(id, (uint8_t*)pub, pubLen,
                                                         (uint8_t*)priv, privLen);

    env->ReleaseStringUTFChars(identity, id);
    env->ReleaseByteArrayElements(pubKey, pub, JNI_ABORT);
    env->ReleaseByteArrayElements(privKey, priv, JNI_ABORT);

    if (handle == nullptr) {
        throwCathedralException(env, CATHEDRAL_ERR_INVALID_KEY);
        return 0;
    }

    return toJavaHandle(handle);
}

JNIEXPORT void JNICALL
Java_cathedral_Agent_nativeMutate(JNIEnv* env, jobject thiz,
                                  jlong handle, jbyteArray mutation, jbyteArray proof) {
    jsize mutLen = env->GetArrayLength(mutation);
    jbyte* mutBytes = env->GetByteArrayElements(mutation, nullptr);
    jsize proofLen = env->GetArrayLength(proof);
    jbyte* proofBytes = env->GetByteArrayElements(proof, nullptr);

    CathedralError err = cathedral_agent_mutate(toNativeHandle(handle),
                                                (uint8_t*)mutBytes, mutLen,
                                                (uint8_t*)proofBytes, proofLen);

    env->ReleaseByteArrayElements(mutation, mutBytes, JNI_ABORT);
    env->ReleaseByteArrayElements(proof, proofBytes, JNI_ABORT);

    if (err != CATHEDRAL_OK) {
        throwCathedralException(env, err);
    }
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_Agent_nativeGetState(JNIEnv* env, jobject thiz, jlong handle) {
    size_t len;
    uint8_t* data = cathedral_agent_get_state(toNativeHandle(handle), &len);
    if (data == nullptr) {
        throwCathedralException(env, CATHEDRAL_ERR_STATE_NOT_FOUND);
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(len);
    env->SetByteArrayRegion(result, 0, len, (jbyte*)data);
    cathedral_free(data);
    return result;
}

// ... outros métodos Agent

// ============================================================================
// Ledger
// ============================================================================

JNIEXPORT jlong JNICALL
Java_cathedral_Ledger_nativeCreate(JNIEnv* env, jclass clazz) {
    CathedralLedgerHandle handle = cathedral_ledger_create();
    return toJavaHandle(handle);
}

JNIEXPORT jboolean JNICALL
Java_cathedral_Ledger_nativeAppend(JNIEnv* env, jobject thiz,
                                   jlong handle, jbyteArray data) {
    jsize len = env->GetArrayLength(data);
    jbyte* bytes = env->GetByteArrayElements(data, nullptr);

    bool result = cathedral_ledger_append(toNativeHandle(handle), (uint8_t*)bytes, len);

    env->ReleaseByteArrayElements(data, bytes, JNI_ABORT);
    return result;
}

// ... outros métodos Ledger

// ============================================================================
// Memory
// ============================================================================

JNIEXPORT jlong JNICALL
Java_cathedral_Memory_nativeCreate(JNIEnv* env, jclass clazz) {
    CathedralMemoryHandle handle = cathedral_memory_create();
    return toJavaHandle(handle);
}

JNIEXPORT void JNICALL
Java_cathedral_Memory_nativeStore(JNIEnv* env, jobject thiz,
                                  jlong handle, jstring key, jbyteArray value,
                                  jint bucket, jlong ttl) {
    const char* keyStr = env->GetStringUTFChars(key, nullptr);
    jsize valLen = env->GetArrayLength(value);
    jbyte* valBytes = env->GetByteArrayElements(value, nullptr);

    cathedral_memory_store(toNativeHandle(handle), keyStr, (uint8_t*)valBytes, valLen,
                          bucket, ttl);

    env->ReleaseStringUTFChars(key, keyStr);
    env->ReleaseByteArrayElements(value, valBytes, JNI_ABORT);
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_Memory_nativeQuery(JNIEnv* env, jobject thiz,
                                  jlong handle, jfloatArray vector,
                                  jint bucket, jint limit, jfloat minSimilarity) {
    jsize vecLen = env->GetArrayLength(vector);
    jfloat* vecData = env->GetFloatArrayElements(vector, nullptr);

    size_t outLen;
    uint8_t* result = cathedral_memory_query(toNativeHandle(handle),
                                             vecData, vecLen,
                                             bucket, limit, minSimilarity,
                                             &outLen);

    env->ReleaseFloatArrayElements(vector, vecData, JNI_ABORT);

    if (result == nullptr) {
        return nullptr;
    }

    jbyteArray jresult = env->NewByteArray(outLen);
    env->SetByteArrayRegion(jresult, 0, outLen, (jbyte*)result);
    cathedral_free(result);
    return jresult;
}

// ... outros métodos

} // extern "C"
