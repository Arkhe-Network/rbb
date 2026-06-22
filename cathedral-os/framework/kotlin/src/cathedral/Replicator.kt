// Replicator.kt
package cathedral

/**
 * Gerenciador de replicação via Nostr.
 * Publica eventos de mutação em relays descentralizados.
 */
class Replicator private constructor(private val nativeHandle: Long) {

    /**
     * Conecta a um conjunto de relays.
     */
    suspend fun connect(relayUrls: List<String>): Boolean {
        return withContext(Dispatchers.IO) {
            nativeConnect(nativeHandle, relayUrls.toTypedArray())
        }
    }

    /**
     * Publica um evento no Nostr.
     * @param event Evento (mutação, governance, etc.)
     * @return ID do evento (hex)
     */
    suspend fun publish(event: NostrEvent): String {
        return withContext(Dispatchers.IO) {
            nativePublish(nativeHandle, event.toBytes())
        }
    }

    /**
     * Inscreve-se para eventos de um agente específico.
     */
    suspend fun subscribe(agentId: String): Flow<NostrEvent> = flow {
        // Implementação com callback JNI
        var subscriptionId = nativeSubscribe(nativeHandle, agentId)
        while (true) {
            val event = nativeNextEvent(nativeHandle, subscriptionId)
            if (event != null) emit(NostrEvent.fromBytes(event))
        }
    }

    companion object {
        private var instance: Replicator? = null

        fun getInstance(): Replicator {
            if (instance == null) {
                val handle = nativeCreate()
                instance = Replicator(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeConnect(handle: Long, relays: Array<String>): Boolean
        private external fun nativePublish(handle: Long, event: ByteArray): String
        private external fun nativeSubscribe(handle: Long, agentId: String): Long
        private external fun nativeNextEvent(handle: Long, subscriptionId: Long): ByteArray?
    }
}

data class NostrEvent(
    val id: String,
    val pubkey: String,
    val kind: Int,
    val content: String,
    val tags: List<List<String>>,
    val sig: String,
    val created_at: Long
) {
    fun toBytes(): ByteArray = json.encodeToString(this).toByteArray()

    companion object {
        fun fromBytes(bytes: ByteArray): NostrEvent {
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}
