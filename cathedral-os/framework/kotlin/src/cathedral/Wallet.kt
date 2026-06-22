// Wallet.kt
package cathedral

/**
 * Carteira para pagamentos L402 (HTTP 402) e Lightning.
 * Permite micropagamentos entre agentes e serviços.
 */
class Wallet private constructor(private val nativeHandle: Long) {

    /**
     * Obtém o saldo atual em milisatoshis.
     */
    suspend fun balance(): Long {
        return withContext(Dispatchers.IO) {
            nativeBalance(nativeHandle)
        }
    }

    /**
     * Cria uma fatura para pagamento.
     * @param amount Valor em milisatoshis
     * @param description Descrição do pagamento
     * @return Fatura (invoice) codificada
     */
    suspend fun createInvoice(amount: Long, description: String): String {
        return withContext(Dispatchers.IO) {
            nativeCreateInvoice(nativeHandle, amount, description)
        }
    }

    /**
     * Paga uma fatura.
     * @param invoice Fatura codificada
     * @return true se o pagamento foi bem-sucedido
     */
    suspend fun pay(invoice: String): Boolean {
        return withContext(Dispatchers.IO) {
            nativePay(nativeHandle, invoice)
        }
    }

    /**
     * Histórico de transações (últimas N).
     */
    suspend fun history(limit: Int = 20): List<Transaction> {
        return withContext(Dispatchers.IO) {
            val bytes = nativeHistory(nativeHandle, limit)
            Transaction.fromBytesList(bytes)
        }
    }

    companion object {
        private var instance: Wallet? = null

        fun getInstance(): Wallet {
            if (instance == null) {
                val handle = nativeCreate()
                instance = Wallet(handle)
            }
            return instance!!
        }

        private external fun nativeCreate(): Long
        private external fun nativeBalance(handle: Long): Long
        private external fun nativeCreateInvoice(handle: Long, amount: Long, description: String): String
        private external fun nativePay(handle: Long, invoice: String): Boolean
        private external fun nativeHistory(handle: Long, limit: Int): ByteArray
    }
}

data class Transaction(
    val id: String,
    val amount: Long,
    val type: TransactionType,
    val timestamp: Long,
    val counterparty: String?,
    val status: TransactionStatus
) {
    companion object {
        fun fromBytesList(bytes: ByteArray): List<Transaction> {
            // Desserialização
            return listOf()
        }
    }
}

enum class TransactionType { INCOMING, OUTGOING, SETTLEMENT }
enum class TransactionStatus { PENDING, COMPLETED, FAILED, SETTLED }
