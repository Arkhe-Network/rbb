// Swarm.kt — Orquestração de múltiplos agentes
package cathedral.swarm

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

class Swarm private constructor(private val nativeHandle: Long) {

    /**
     * Cria um enxame com um conjunto de agentes.
     */
    suspend fun createSwarm(agents: List<String>, config: SwarmConfig): SwarmId {
        return withContext(Dispatchers.IO) {
            val id = nativeCreateSwarm(nativeHandle, agents.toTypedArray(), config.toBytes())
            SwarmId(id)
        }
    }

    /**
     * Envia uma mensagem para todos os agentes do enxame.
     */
    suspend fun broadcast(swarmId: SwarmId, message: SwarmMessage): Boolean {
        return withContext(Dispatchers.IO) {
            nativeBroadcast(nativeHandle, swarmId.id, message.toBytes())
        }
    }

    /**
     * Executa uma votação de consenso no enxame (BFT).
     */
    suspend fun consensusVote(swarmId: SwarmId, proposal: Proposal): VoteResult {
        return withContext(Dispatchers.IO) {
            val result = nativeConsensusVote(nativeHandle, swarmId.id, proposal.toBytes())
            VoteResult.fromBytes(result)
        }
    }

    /**
     * Fluxo de eventos do enxame.
     */
    fun events(swarmId: SwarmId): Flow<SwarmEvent> = flow {
        var cursor = 0L
        while (true) {
            val event = nativeNextEvent(nativeHandle, swarmId.id, cursor)
            if (event == null) break
            emit(SwarmEvent.fromBytes(event))
            cursor++
        }
    }

    companion object {
        private var instance: Swarm? = null

        fun getInstance(): Swarm {
            if (instance == null) {
                val handle = nativeCreate()
                instance = Swarm(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeCreateSwarm(handle: Long, agents: Array<String>, config: ByteArray): String
        private external fun nativeBroadcast(handle: Long, swarmId: String, message: ByteArray): Boolean
        private external fun nativeConsensusVote(handle: Long, swarmId: String, proposal: ByteArray): ByteArray
        private external fun nativeNextEvent(handle: Long, swarmId: String, cursor: Long): ByteArray?
    }
}

data class SwarmConfig(
    val consensus: ConsensusType = ConsensusType.PBFT,
    val quorum: Int = 2,
    val timeoutMs: Long = 5000,
    val maxAgents: Int = 100
) {
    fun toBytes(): ByteArray = json.encodeToString(this).toByteArray()
}

enum class ConsensusType { PBFT, RAFT, POOL }
