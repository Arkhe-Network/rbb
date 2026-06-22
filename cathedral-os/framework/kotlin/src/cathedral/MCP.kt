// MCP.kt — Model Context Protocol Server
package cathedral.mcp

import kotlinx.coroutines.flow.Flow

/**
 * Servidor MCP que expõe funções do agente para LLMs.
 * Compatível com o protocolo Model Context Protocol (Anthropic).
 */
class MCPServer private constructor(private val nativeHandle: Long) {

    /**
     * Inicia o servidor MCP em uma porta local.
     */
    suspend fun start(port: Int = 8080): Boolean {
        return withContext(Dispatchers.IO) {
            nativeStart(nativeHandle, port)
        }
    }

    /**
     * Registra uma ferramenta (função) no servidor MCP.
     */
    suspend fun registerTool(
        name: String,
        description: String,
        handler: (ByteArray) -> ByteArray
    ): Boolean {
        return withContext(Dispatchers.IO) {
            val handlerId = nativeRegisterHandler(nativeHandle, name, description)
            // Armazena o handler para callback JNI
            toolHandlers[handlerId] = handler
            true
        }
    }

    /**
     * Fluxo de requisições recebidas via MCP.
     */
    fun requests(): Flow<MCPRequest> = flow {
        while (true) {
            val req = nativeNextRequest(nativeHandle)
            if (req == null) break
            emit(MCPRequest.fromBytes(req))
        }
    }

    /**
     * Responde a uma requisição MCP.
     */
    suspend fun respond(requestId: String, response: MCPResponse): Boolean {
        return withContext(Dispatchers.IO) {
            nativeRespond(nativeHandle, requestId, response.toBytes())
        }
    }

    companion object {
        private var instance: MCPServer? = null
        private val toolHandlers = mutableMapOf<Long, (ByteArray) -> ByteArray>()

        fun getInstance(): MCPServer {
            if (instance == null) {
                val handle = nativeCreate()
                instance = MCPServer(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeStart(handle: Long, port: Int): Boolean
        private external fun nativeRegisterHandler(handle: Long, name: String, description: String): Long
        private external fun nativeNextRequest(handle: Long): ByteArray?
        private external fun nativeRespond(handle: Long, requestId: String, response: ByteArray): Boolean
    }
}

data class MCPRequest(
    val id: String,
    val tool: String,
    val parameters: Map<String, Any>,
    val timestamp: Long
) {
    fun toBytes(): ByteArray = json.encodeToString(this).toByteArray()
    companion object {
        fun fromBytes(bytes: ByteArray): MCPRequest {
            return json.decodeFromString(bytes.decodeToString())
        }
    }
}

data class MCPResponse(
    val id: String,
    val result: Any,
    val error: String? = null
) {
    fun toBytes(): ByteArray = json.encodeToString(this).toByteArray()
}
