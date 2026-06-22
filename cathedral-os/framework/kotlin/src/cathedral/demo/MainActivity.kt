// MainActivity.kt — Versão completa com todas as features
package cathedral.demo

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import cathedral.*
import cathedral.pqc.*
import cathedral.swarm.*
import cathedral.fl.*
import cathedral.mcp.*
import cathedral.security.*

class MainActivity : ComponentActivity() {
    private lateinit var agent: AgentPQC
    private lateinit var ledger: Ledger
    private lateinit var memory: Memory
    private lateinit var wallet: Wallet
    private lateinit var swarm: Swarm
    private lateinit var fl: FederatedLearning
    private lateinit var mcp: MCPServer
    private lateinit var anomaly: AnomalyDetection
    private lateinit var pqc: PQC

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Inicializa serviços
        val serviceIntent = Intent(this, CathedralService::class.java)
        startService(serviceIntent)

        // Obtém instâncias
        pqc = PQC.getInstance()
        ledger = Ledger.getInstance()
        memory = Memory.getInstance()
        wallet = Wallet.getInstance()
        swarm = Swarm.getInstance()
        fl = FederatedLearning.getInstance()
        mcp = MCPServer.getInstance()
        anomaly = AnomalyDetection.getInstance()

        // Gera chaves PQC
        lifecycleScope.launch {
            try {
                // 1. Gera par SLH-DSA (SPHINCS+) para o agente
                val slhDsaKeyPair = pqc.generateSLHDSAPair(256)

                // 2. Cria agente com identidade PQC
                agent = AgentPQC.createPQC("demo_agent_pqc", slhDsaKeyPair)

                // 3. Gera certificado quântico-seguro
                val certificate = pqc.generateQuantumSafeCertificate(
                    agentId = "demo_agent_pqc",
                    publicKey = slhDsaKeyPair.public.encoded,
                    caPrivateKey = getCAPrivateKey() // Carregada do keystore
                )

                // 4. Verifica certificado
                val valid = pqc.verifyQuantumSafeCertificate(certificate)
                if (valid) {
                    Toast.makeText(this@MainActivity, "Certificado PQC válido!", Toast.LENGTH_SHORT).show()
                }

                // Exibe estado
                val state = agent.getState()
                findViewById<TextView>(R.id.textState).text = """
                    Agente PQC: ${agent.identity}
                    Certificado: ${certificate.id}
                    Válido até: ${certificate.validUntil}
                """.trimIndent()

            } catch (e: Exception) {
                Toast.makeText(this@MainActivity, "Erro PQC: ${e.message}", Toast.LENGTH_LONG).show()
            }
        }

        // Botão: Mutar com assinatura SLH-DSA (PQC)
        findViewById<Button>(R.id.btnMutatePQC).setOnClickListener {
            lifecycleScope.launch {
                try {
                    val mutation = Mutation(
                        field = "status",
                        value = "ativo_pqc",
                        timestamp = System.currentTimeMillis()
                    )

                    // Carrega chave privada SLH-DSA
                    val privateKey = getPrivateKey() // Do keystore

                    // Muta com assinatura PQC
                    agent.mutatePQC(mutation, privateKey)

                    // Registra no ledger com hash PQC
                    ledger.append(
                        ProvenanceEntry(
                            id = UUID.randomUUID().toString(),
                            version = 1,
                            decisionType = "mutation_pqc",
                            beforeState = "{}",
                            afterState = "{\"status\": \"ativo_pqc\"}",
                            rationale = "Mutação assinada com SLH-DSA",
                            timestamp = System.currentTimeMillis(),
                            agentId = agent.identity,
                            entryHash = mutation.toBytes().sha3(),
                            nostrEventId = null,
                            treeId = null,
                            parentEventId = null
                        )
                    )

                    Toast.makeText(this@MainActivity, "Mutação PQC realizada!", Toast.LENGTH_SHORT).show()
                } catch (e: CathedralException) {
                    Toast.makeText(this@MainActivity, "Erro PQC: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }

        // Botão: Estabelecer sessão segura PQC (ML-KEM)
        findViewById<Button>(R.id.btnSecureSession).setOnClickListener {
            lifecycleScope.launch {
                try {
                    // Gera par ML-KEM para o alvo (simulado)
                    val targetKeyPair = pqc.generateMLKEMPair()

                    // Estabelece sessão segura
                    val sessionKey = agent.establishSecureSession(targetKeyPair.publicKey)

                    Toast.makeText(
                        this@MainActivity,
                        "Sessão PQC estabelecida! Chave: ${sessionKey.take(16).toHex()}...",
                        Toast.LENGTH_LONG
                    ).show()

                    // Salva a chave de sessão no memory M1 (cache)
                    memory.store(
                        key = "session_key_${System.currentTimeMillis()}",
                        value = sessionKey,
                        bucket = MemoryBucket.M1,
                        ttlSeconds = 3600
                    )
                } catch (e: Exception) {
                    Toast.makeText(this@MainActivity, "Erro: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }

        // Botão: Criar enxame
        findViewById<Button>(R.id.btnCreateSwarm).setOnClickListener {
            lifecycleScope.launch {
                try {
                    val agents = listOf(
                        "agent_1", "agent_2", "agent_3", "agent_4"
                    )
                    val config = SwarmConfig(
                        consensus = ConsensusType.PBFT,
                        quorum = 3,
                        timeoutMs = 10000
                    )
                    val swarmId = swarm.createSwarm(agents, config)

                    // Vota em uma proposta
                    val proposal = Proposal(
                        id = "proposal_001",
                        description = "Aumentar limite de mutações",
                        vote = VoteType.APPROVE
                    )
                    val result = swarm.consensusVote(swarmId, proposal)

                    Toast.makeText(
                        this@MainActivity,
                        "Enxame criado! Votação: ${result.approved} (${result.votes}/${result.quorum})",
                        Toast.LENGTH_LONG
                    ).show()
                } catch (e: Exception) {
                    Toast.makeText(this@MainActivity, "Erro: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }

        // Botão: Federated Learning
        findViewById<Button>(R.id.btnFL).setOnClickListener {
            lifecycleScope.launch {
                try {
                    val modelId = fl.initModel(ModelType.NEURAL_NETWORK, FLConfig(
                        epochs = 10,
                        batchSize = 32,
                        learningRate = 0.001f
                    ))

                    // Dados de treino (mock)
                    val trainingData = byteArrayOf(1, 2, 3, 4, 5)
                    val loss = fl.trainLocal(modelId, trainingData)

                    // Compartilha gradientes
                    fl.shareGradients(modelId, listOf("agent_2", "agent_3"))

                    Toast.makeText(
                        this@MainActivity,
                        "FL: Loss=${loss}, Modelo ${modelId.id}",
                        Toast.LENGTH_LONG
                    ).show()
                } catch (e: Exception) {
                    Toast.makeText(this@MainActivity, "Erro FL: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }

        // Botão: MCP Server
        findViewById<Button>(R.id.btnMCP).setOnClickListener {
            lifecycleScope.launch {
                try {
                    val started = mcp.start(8080)
                    if (started) {
                        mcp.registerTool("get_agent_status", "Obtém o status do agente") { params ->
                            // Handler MCP
                            val state = agent.getState()
                            json.encodeToString(state).toByteArray()
                        }

                        Toast.makeText(
                            this@MainActivity,
                            "MCP Server rodando em localhost:8080",
                            Toast.LENGTH_LONG
                        ).show()
                    }
                } catch (e: Exception) {
                    Toast.makeText(this@MainActivity, "Erro MCP: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }

        // Botão: Anomaly Detection
        findViewById<Button>(R.id.btnAnomaly).setOnClickListener {
            lifecycleScope.launch {
                try {
                    val reports = anomaly.scanAllAgents()
                    val text = reports.joinToString("\n") {
                        "Agente ${it.agentId}: score=${it.score}, anomalias=${it.anomalies}"
                    }
                    Toast.makeText(
                        this@MainActivity,
                        "Detecção de anomalias:\n$text",
                        Toast.LENGTH_LONG
                    ).show()
                } catch (e: Exception) {
                    Toast.makeText(this@MainActivity, "Erro: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }
    }

    override fun onDestroy() {
        agent.close()
        super.onDestroy()
    }

    private fun getCAPrivateKey(): ByteArray {
        // Carrega a chave privada da CA do keystore
        return KeyManager.loadPrivateKey("ca_slhdsa").encoded
    }

    private fun getPrivateKey(): ByteArray {
        // Carrega a chave privada do agente
        return KeyManager.loadPrivateKey("demo_agent_pqc").encoded
    }
}
