// Agent.kt
package cathedral

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.security.KeyPair

/**
 * Representa um agente autônomo com identidade, memória e capacidade de mutação.
 *
 * @property identity Identificador único do agente (DID)
 * @property nativeHandle Ponteiro para o objeto Rust (JNI)
 */
class Agent(private val nativeHandle: Long) {
    val identity: String = nativeGetIdentity(nativeHandle)
    val reputation: Int = nativeGetReputation(nativeHandle)

    /**
     * Muta o agente com uma nova ação, acompanhada de prova ZK.
     * @param mutation Dados da mutação (ex: novo estado, decisão)
     * @param proof Prova ZK de segurança/ética
     */
    suspend fun mutate(mutation: Mutation, proof: ZkProof): Result<Unit> {
        return withContext(Dispatchers.IO) {
            try {
                nativeMutate(nativeHandle, mutation.toBytes(), proof.toBytes())
                Result.success(Unit)
            } catch (e: CathedralException) {
                Result.failure(e)
            }
        }
    }

    /**
     * Obtém o estado atual do agente.
     */
    suspend fun getState(): State {
        return withContext(Dispatchers.IO) {
            State.fromBytes(nativeGetState(nativeHandle))
        }
    }

    /**
     * Fluxo de eventos do agente (mutações observadas).
     */
    fun events(): Flow<Event> = flow {
        while (true) {
            val event = nativeNextEvent(nativeHandle)
            if (event != null) emit(Event.fromBytes(event))
            delay(100) // Polling leve
        }
    }

    /**
     * Libera recursos nativos.
     */
    fun close() {
        if (nativeHandle != 0L) nativeDestroy(nativeHandle)
    }

    companion object {
        init {
            System.loadLibrary("cathedral") // Carrega libcathedral.so
        }

        /**
         * Cria um novo agente com identidade e chave Ed25519.
         */
        fun create(identity: String, keyPair: KeyPair): Agent {
            val handle = nativeCreateAgent(identity, keyPair.public.encoded, keyPair.private.encoded)
            return Agent(handle)
        }

        private external fun nativeCreateAgent(
            identity: String,
            publicKey: ByteArray,
            privateKey: ByteArray
        ): Long

        private external fun nativeMutate(handle: Long, mutation: ByteArray, proof: ByteArray)
        private external fun nativeGetState(handle: Long): ByteArray
        private external fun nativeGetIdentity(handle: Long): String
        private external fun nativeGetReputation(handle: Long): Int
        private external fun nativeNextEvent(handle: Long): ByteArray?
        private external fun nativeDestroy(handle: Long)
    }
}

data class Mutation(
    val field: String,
    val value: Any,
    val timestamp: Long = System.currentTimeMillis()
) {
    fun toBytes(): ByteArray {
        // Serialização para Rust (JSON, Protobuf ou CBOR)
        return json.encodeToString(this).toByteArray()
    }
}

data class State(
    val id: String,
    val version: Int,
    val reputation: Int,
    val data: Map<String, Any>
) {
    companion object {
        fun fromBytes(bytes: ByteArray): State {
            // Desserialização de bytes para State
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}
