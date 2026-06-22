// FederatedLearning.kt
package cathedral.fl

class FederatedLearning private constructor(private val nativeHandle: Long) {

    /**
     * Inicializa um modelo de aprendizado federado.
     */
    suspend fun initModel(modelType: ModelType, config: FLConfig): ModelId {
        return withContext(Dispatchers.IO) {
            val id = nativeInitModel(nativeHandle, modelType.name, config.toBytes())
            ModelId(id)
        }
    }

    /**
     * Treina o modelo localmente com dados do agente.
     */
    suspend fun trainLocal(modelId: ModelId, data: ByteArray): Float {
        return withContext(Dispatchers.IO) {
            nativeTrainLocal(nativeHandle, modelId.id, data)
        }
    }

    /**
     * Envia gradientes para o aggregator (Nostr ou P2P).
     */
    suspend fun shareGradients(modelId: ModelId, targetAgents: List<String>): Boolean {
        return withContext(Dispatchers.IO) {
            nativeShareGradients(nativeHandle, modelId.id, targetAgents.toTypedArray())
        }
    }

    /**
     * Agrega gradientes recebidos de outros agentes.
     */
    suspend fun aggregateGradients(modelId: ModelId): Float {
        return withContext(Dispatchers.IO) {
            nativeAggregateGradients(nativeHandle, modelId.id)
        }
    }

    companion object {
        private var instance: FederatedLearning? = null

        fun getInstance(): FederatedLearning {
            if (instance == null) {
                val handle = nativeCreate()
                instance = FederatedLearning(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeInitModel(handle: Long, modelType: String, config: ByteArray): String
        private external fun nativeTrainLocal(handle: Long, modelId: String, data: ByteArray): Float
        private external fun nativeShareGradients(handle: Long, modelId: String, targets: Array<String>): Boolean
        private external fun nativeAggregateGradients(handle: Long, modelId: String): Float
    }
}

enum class ModelType { NEURAL_NETWORK, DECISION_TREE, RANDOM_FOREST, TRANSFORMER }
