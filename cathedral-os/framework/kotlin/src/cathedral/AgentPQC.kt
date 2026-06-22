// AgentPQC.kt
package cathedral.pqc

class AgentPQC(private val nativeHandle: Long) {
    private val identity: String = nativeGetIdentity(nativeHandle)
    private val pqc = PQC.getInstance()

    /**
     * Muta o agente com assinatura PQC (SLH-DSA).
     */
    suspend fun mutatePQC(mutation: Mutation, privateKey: PrivateKey): Result<Unit> {
        return withContext(Dispatchers.IO) {
            try {
                // Serializa a mutação
                val mutationBytes = mutation.toBytes()

                // Assina com SLH-DSA (PQC)
                val signature = pqc.signSLHDSA(mutationBytes, privateKey)

                // Envia mutação + assinatura para o núcleo Rust
                nativeMutatePQC(nativeHandle, mutationBytes, signature)
                Result.success(Unit)
            } catch (e: CathedralException) {
                Result.failure(e)
            }
        }
    }

    /**
     * Verifica uma mutação recebida (com assinatura PQC).
     */
    suspend fun verifyMutationPQC(mutation: Mutation, signature: ByteArray, publicKey: PublicKey): Boolean {
        return withContext(Dispatchers.IO) {
            pqc.verifySLHDSA(mutation.toBytes(), signature, publicKey)
        }
    }

    /**
     * Estabelece uma sessão segura com outro agente usando ML-KEM (Kyber).
     */
    suspend fun establishSecureSession(targetPublicKey: ByteArray): ByteArray {
        return withContext(Dispatchers.IO) {
            // Gera par efêmero ML-KEM
            val ephemeralPair = pqc.generateMLKEMPair()

            // Encapsula o segredo compartilhado
            val (ciphertext, sharedSecret) = pqc.encapsulateMLKEM(targetPublicKey)

            // Envia ciphertext para o target e recebe confirmação
            val sessionKey = nativeExchangeSession(nativeHandle, ciphertext, ephemeralPair.privateKey)

            sessionKey // Retorna a chave de sessão estabelecida (PQC-safe)
        }
    }

    companion object {
        init {
            System.loadLibrary("cathedral")
        }

        fun createPQC(identity: String, slhDsaKeyPair: KeyPair): AgentPQC {
            val handle = nativeCreateAgentPQC(identity, slhDsaKeyPair.public.encoded, slhDsaKeyPair.private.encoded)
            return AgentPQC(handle)
        }

        private external fun nativeCreateAgentPQC(
            identity: String,
            publicKey: ByteArray,
            privateKey: ByteArray
        ): Long

        private external fun nativeMutatePQC(handle: Long, mutation: ByteArray, signature: ByteArray)
        private external fun nativeExchangeSession(handle: Long, ciphertext: ByteArray, ephemeralPrivate: ByteArray): ByteArray
        private external fun nativeGetIdentity(handle: Long): String
    }
}
