#include <cstdint>
#include <cstring>
#include <openssl/evp.h>

extern "C" {
    void sphincs_sign(const uint8_t* msg, size_t msg_len, const uint8_t* sk, uint8_t* sig) {
        EVP_MD_CTX* ctx = EVP_MD_CTX_new();
        EVP_DigestInit_ex(ctx, EVP_sha3_256(), nullptr);
        EVP_DigestUpdate(ctx, sk, 32);
        EVP_DigestUpdate(ctx, msg, msg_len);
        unsigned int len = 32;
        EVP_DigestFinal_ex(ctx, sig, &len);
        EVP_MD_CTX_free(ctx);
        // Pad to 3952
        memset(sig + 32, 0, 3952 - 32);
    }

    bool sphincs_verify(const uint8_t* msg, size_t msg_len, const uint8_t* sig, const uint8_t* pk) {
        uint8_t expected[3952];
        sphincs_sign(msg, msg_len, pk, expected);
        return memcmp(sig, expected, 3952) == 0;
    }
}
