// CathedralService.kt
package cathedral

import android.app.Service
import android.content.Intent
import android.os.IBinder
import kotlinx.coroutines.*

class CathedralService : Service() {
    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private lateinit var agent: Agent
    private lateinit var ledger: Ledger
    private lateinit var memory: Memory
    private lateinit var wallet: Wallet
    private lateinit var replicator: Replicator

    override fun onCreate() {
        super.onCreate()

        // Inicializa runtime Cathedral
        CathedralRuntime.initialize(applicationContext)

        // Carrega ou gera chave do Keystore
        val keyPair = KeyManager.loadOrCreate("default_agent")

        // Cria agente principal
        agent = Agent.create("default_agent", keyPair)
        ledger = Ledger.getInstance()
        memory = Memory.getInstance()
        wallet = Wallet.getInstance()
        replicator = Replicator.getInstance()

        // Conecta aos relays
        serviceScope.launch {
            replicator.connect(listOf(
                "wss://relay.damus.io",
                "wss://nos.lol",
                "wss://relay.snort.social"
            ))

            // Escuta eventos do agente
            agent.events().collect { event ->
                // Publica mutações no Nostr
                replicator.publish(
                    NostrEvent(
                        id = event.id,
                        pubkey = keyPair.public.toHex(),
                        kind = 30078,
                        content = event.payload,
                        tags = listOf(
                            listOf("agent", agent.identity),
                            listOf("type", event.type)
                        ),
                        sig = "",
                        created_at = System.currentTimeMillis() / 1000
                    )
                )

                // Registra no ledger local
                ledger.append(
                    ProvenanceEntry(
                        id = event.id,
                        version = 1,
                        decisionType = event.type,
                        beforeState = "",
                        afterState = event.payload,
                        rationale = event.rationale,
                        timestamp = System.currentTimeMillis(),
                        agentId = agent.identity,
                        entryHash = event.hash,
                        nostrEventId = null,
                        treeId = null,
                        parentEventId = null
                    )
                )
            }
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }

    override fun onDestroy() {
        serviceScope.cancel()
        agent.close()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
