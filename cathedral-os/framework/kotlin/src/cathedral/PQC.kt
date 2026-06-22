// PQC.kt — Post-Quantum Cryptography
package cathedral.pqc

import java.security.KeyPair
import java.security.PrivateKey
import java.security.PublicKey

/**
 * Gerenciador de criptografia pós-quântica para Cathedral-OS.
 * Suporta SLH-DSA (SPHINCS+) e ML-KEM (Kyber).
 */
class PQC private constructor(private val nativeHandle: Long) {

    /**
     * Gera um par de chaves SLH-DSA (SPHINCS+) para assinaturas.
     * @param securityLevel Nível de segurança: 128, 192, 256
     * @return Par de chaves com SLH-DSA
     */
    suspend fun generateSLHDSAPair(securityLevel: Int = 256): KeyPair {
        return withContext(Dispatchers.IO) {
            val bytes = nativeGenerateSLHDSAPair(nativeHandle, securityLevel)
            KeyPair(
                PublicKey.fromBytes(bytes.sliceArray(0..pubKeyLen)),
                PrivateKey.fromBytes(bytes.sliceArray(pubKeyLen until bytes.size))
            )
        }
    }

    /**
     * Assina uma mensagem com SLH-DSA.
     * @param message Mensagem a ser assinada
     * @param privateKey Chave privada SLH-DSA
     * @return Assinatura
     */
    suspend fun signSLHDSA(message: ByteArray, privateKey: PrivateKey): ByteArray {
        return withContext(Dispatchers.IO) {
            nativeSignSLHDSA(nativeHandle, message, privateKey.encoded)
        }
    }

    /**
     * Verifica uma assinatura SLH-DSA.
     * @param message Mensagem original
     * @param signature Assinatura
     * @param publicKey Chave pública
     * @return true se a assinatura for válida
     */
    suspend fun verifySLHDSA(message: ByteArray, signature: ByteArray, publicKey: PublicKey): Boolean {
        return withContext(Dispatchers.IO) {
            nativeVerifySLHDSA(nativeHandle, message, signature, publicKey.encoded)
        }
    }

    /**
     * Gera um par de chaves ML-KEM (Kyber) para troca de chaves.
     * @param securityLevel Nível de segurança: 512, 768, 1024
     * @return Par de chaves ML-KEM
     */
    suspend fun generateMLKEMPair(securityLevel: Int = 768): MLKEMKeyPair {
        return withContext(Dispatchers.IO) {
            val bytes = nativeGenerateMLKEMPair(nativeHandle, securityLevel)
            MLKEMKeyPair(
                publicKey = bytes.sliceArray(0 until publicKeyLen),
                privateKey = bytes.sliceArray(publicKeyLen until bytes.size)
            )
        }
    }

    /**
     * Encapsula uma chave usando ML-KEM.
     * @param publicKey Chave pública do receptor
     * @return Par (ciphertext, sharedSecret)
     */
    suspend fun encapsulateMLKEM(publicKey: ByteArray): Pair<ByteArray, ByteArray> {
        return withContext(Dispatchers.IO) {
            val result = nativeEncapsulateMLKEM(nativeHandle, publicKey)
            Pair(
                result.sliceArray(0 until ciphertextLen),
                result.sliceArray(ciphertextLen until result.size)
            )
        }
    }

    /**
     * Decapsula uma chave usando ML-KEM.
     * @param ciphertext Texto cifrado
     * @param privateKey Chave privada do receptor
     * @return Segredo compartilhado
     */
    suspend fun decapsulateMLKEM(ciphertext: ByteArray, privateKey: ByteArray): ByteArray {
        return withContext(Dispatchers.IO) {
            nativeDecapsulateMLKEM(nativeHandle, ciphertext, privateKey)
        }
    }

    /**
     * Gera um certificado quântico-seguro (QSC) para um agente.
     * @param agentId ID do agente
     * @param publicKey Chave pública SLH-DSA
     * @param caPrivateKey Chave privada da CA (SLH-DSA)
     * @return Certificado QSC assinado
     */
    suspend fun generateQuantumSafeCertificate(
        agentId: String,
        publicKey: ByteArray,
        caPrivateKey: ByteArray
    ): QuantumSafeCertificate {
        return withContext(Dispatchers.IO) {
            val bytes = nativeGenerateQSC(nativeHandle, agentId, publicKey, caPrivateKey)
            QuantumSafeCertificate.fromBytes(bytes)
        }
    }

    /**
     * Verifica um certificado quântico-seguro.
     */
    suspend fun verifyQuantumSafeCertificate(certificate: QuantumSafeCertificate): Boolean {
        return withContext(Dispatchers.IO) {
            nativeVerifyQSC(nativeHandle, certificate.toBytes())
        }
    }

    companion object {
        private var instance: PQC? = null

        fun getInstance(): PQC {
            if (instance == null) {
                val handle = nativeCreate()
                instance = PQC(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeGenerateSLHDSAPair(handle: Long, level: Int): ByteArray
        private external fun nativeSignSLHDSA(handle: Long, message: ByteArray, privateKey: ByteArray): ByteArray
        private external fun nativeVerifySLHDSA(handle: Long, message: ByteArray, signature: ByteArray, publicKey: ByteArray): Boolean
        private external fun nativeGenerateMLKEMPair(handle: Long, level: Int): ByteArray
        private external fun nativeEncapsulateMLKEM(handle: Long, publicKey: ByteArray): ByteArray
        private external fun nativeDecapsulateMLKEM(handle: Long, ciphertext: ByteArray, privateKey: ByteArray): ByteArray
        private external fun nativeGenerateQSC(handle: Long, agentId: String, publicKey: ByteArray, caPrivateKey: ByteArray): ByteArray
        private external fun nativeVerifyQSC(handle: Long, certificate: ByteArray): Boolean
    }
}

/**
 * Certificado Quântico-Seguro (QSC).
 */
data class QuantumSafeCertificate(
    val id: String,
    val agentId: String,
    val publicKey: ByteArray,
    val signature: ByteArray,
    val issuer: String,
    val validFrom: Long,
    val validUntil: Long,
    val extensions: Map<String, ByteArray>
) {
    fun toBytes(): ByteArray {
        return json.encodeToString(this).toByteArray()
    }

    companion object {
        fun fromBytes(bytes: ByteArray): QuantumSafeCertificate {
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}

data class MLKEMKeyPair(
    val publicKey: ByteArray,
    val privateKey: ByteArray
)
