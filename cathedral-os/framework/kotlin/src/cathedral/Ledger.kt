// Ledger.kt
package cathedral

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Ledger imutável baseado em WormGraph.
 * Armazena proveniência de todas as ações.
 */
class Ledger private constructor(private val nativeHandle: Long) {

    /**
     * Adiciona uma entrada ao ledger.
     * @param entry Entrada de proveniência (mutação, permissão, transação)
     * @return true se bem-sucedido
     */
    suspend fun append(entry: ProvenanceEntry): Boolean {
        return withContext(Dispatchers.IO) {
            nativeAppend(nativeHandle, entry.toBytes())
        }
    }

    /**
     * Busca uma entrada pelo hash.
     */
    suspend fun query(hash: ByteArray): ProvenanceEntry? {
        return withContext(Dispatchers.IO) {
            val bytes = nativeQuery(nativeHandle, hash)
            bytes?.let { ProvenanceEntry.fromBytes(it) }
        }
    }

    /**
     * Verifica a integridade de uma entrada.
     */
    suspend fun verify(entry: ProvenanceEntry): Boolean {
        return withContext(Dispatchers.IO) {
            nativeVerify(nativeHandle, entry.toBytes())
        }
    }

    /**
     * Fluxo de todas as entradas do ledger (streaming).
     */
    fun stream(): Flow<ProvenanceEntry> = flow {
        var cursor = 0L
        while (true) {
            val entry = nativeNextEntry(nativeHandle, cursor)
            if (entry == null) break
            emit(ProvenanceEntry.fromBytes(entry))
            cursor++
        }
    }

    /**
     * Obtém a raiz Merkle atual do ledger.
     */
    suspend fun getMerkleRoot(): ByteArray {
        return withContext(Dispatchers.IO) {
            nativeGetMerkleRoot(nativeHandle)
        }
    }

    companion object {
        private var instance: Ledger? = null

        fun getInstance(): Ledger {
            if (instance == null) {
                val handle = nativeCreate()
                instance = Ledger(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeAppend(handle: Long, data: ByteArray): Boolean
        private external fun nativeQuery(handle: Long, hash: ByteArray): ByteArray?
        private external fun nativeVerify(handle: Long, data: ByteArray): Boolean
        private external fun nativeNextEntry(handle: Long, cursor: Long): ByteArray?
        private external fun nativeGetMerkleRoot(handle: Long): ByteArray
    }
}

data class ProvenanceEntry(
    val id: String,
    val version: Int,
    val decisionType: String,
    val beforeState: String,
    val afterState: String,
    val rationale: String?,
    val timestamp: Long,
    val agentId: String,
    val entryHash: ByteArray,
    val nostrEventId: String?,
    val treeId: String?,
    val parentEventId: String?
) {
    fun toBytes(): ByteArray = json.encodeToString(this).toByteArray()

    companion object {
        fun fromBytes(bytes: ByteArray): ProvenanceEntry {
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}
