// Memory.kt
package cathedral

enum class MemoryBucket {
    M0, M1, M2, M3, M4
}

data class MemoryItem(
    val key: String,
    val value: ByteArray,
    val bucket: MemoryBucket,
    val ttl: Long,
    val vector: FloatArray
)

/**
 * Gerenciador de memória semântica (Plurality buckets M0-M4).
 * Substitui SharedPreferences com busca por similaridade.
 */
class Memory private constructor(private val nativeHandle: Long) {

    /**
     * Armazena um valor em um bucket específico.
     * @param key Chave única
     * @param value Valor (bytes)
     * @param bucket Bucket de memória (M0-M4)
     * @param ttlSeconds Tempo de vida em segundos (0 = infinito)
     */
    suspend fun store(key: String, value: ByteArray, bucket: MemoryBucket, ttlSeconds: Long = 0) {
        withContext(Dispatchers.IO) {
            nativeStore(nativeHandle, key, value, bucket.ordinal, ttlSeconds)
        }
    }

    /**
     * Recupera um valor pelo bucket e chave.
     */
    suspend fun retrieve(key: String, bucket: MemoryBucket): ByteArray? {
        return withContext(Dispatchers.IO) {
            nativeRetrieve(nativeHandle, key, bucket.ordinal)
        }
    }

    /**
     * Busca por similaridade semântica (vetor) em um bucket.
     * @param vector Vetor de embedding
     * @param bucket Bucket alvo
     * @param limit Número máximo de resultados
     * @param minSimilarity Similaridade mínima (0.0 a 1.0)
     */
    suspend fun query(
        vector: FloatArray,
        bucket: MemoryBucket,
        limit: Int = 10,
        minSimilarity: Float = 0.7f
    ): List<MemoryItem> {
        return withContext(Dispatchers.IO) {
            val bytes = nativeQuery(nativeHandle, vector, bucket.ordinal, limit, minSimilarity)
            MemoryItem.fromBytesList(bytes)
        }
    }

    /**
     * Compartilha memória com outro agente (M3).
     */
    suspend fun share(key: String, targetAgentId: String, bucket: MemoryBucket = MemoryBucket.M3) {
        withContext(Dispatchers.IO) {
            nativeShare(nativeHandle, key, targetAgentId, bucket.ordinal)
        }
    }

    companion object {
        private var instance: Memory? = null

        fun getInstance(): Memory {
            if (instance == null) {
                val handle = nativeCreate()
                instance = Memory(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeStore(handle: Long, key: String, value: ByteArray, bucket: Int, ttl: Long)
        private external fun nativeRetrieve(handle: Long, key: String, bucket: Int): ByteArray?
        private external fun nativeQuery(handle: Long, vector: FloatArray, bucket: Int, limit: Int, minSimilarity: Float): ByteArray
        private external fun nativeShare(handle: Long, key: String, targetId: String, bucket: Int)
    }
}
