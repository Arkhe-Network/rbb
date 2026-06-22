// AnomalyDetection.kt
package cathedral.security

class AnomalyDetection private constructor(private val nativeHandle: Long) {

    /**
     * Treina o modelo de detecção de anomalias com dados históricos.
     */
    suspend fun train(historicalData: List<AgentBehavior>): Float {
        return withContext(Dispatchers.IO) {
            val serialized = json.encodeToString(historicalData)
            nativeTrain(nativeHandle, serialized.toByteArray())
        }
    }

    /**
     * Detecta anomalias no comportamento atual de um agente.
     */
    suspend fun detect(behavior: AgentBehavior): AnomalyReport {
        return withContext(Dispatchers.IO) {
            val bytes = nativeDetect(nativeHandle, json.encodeToString(behavior).toByteArray())
            AnomalyReport.fromBytes(bytes)
        }
    }

    /**
     * Escaneia periodicamente todos os agentes locais.
     */
    suspend fun scanAllAgents(): List<AnomalyReport> {
        return withContext(Dispatchers.IO) {
            val bytes = nativeScanAll(nativeHandle)
            json.decodeFromString(bytes.decodeToString())
        }
    }

    companion object {
        private var instance: AnomalyDetection? = null

        fun getInstance(): AnomalyDetection {
            if (instance == null) {
                val handle = nativeCreate()
                instance = AnomalyDetection(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeTrain(handle: Long, data: ByteArray): Float
        private external fun nativeDetect(handle: Long, behavior: ByteArray): ByteArray
        private external fun nativeScanAll(handle: Long): ByteArray
    }
}

data class AgentBehavior(
    val agentId: String,
    val mutationRate: Float,
    val energyConsumption: Float,
    val memoryUsage: Float,
    val interactionCount: Int,
    val reputation: Int,
    val timestamp: Long
)

data class AnomalyReport(
    val agentId: String,
    val score: Float,
    val threshold: Float,
    val anomalies: List<String>,
    val recommendations: List<String>,
    val timestamp: Long
) {
    companion object {
        fun fromBytes(bytes: ByteArray): AnomalyReport {
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}
