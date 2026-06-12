# Within knowledge-base service or module

import json

def store_document(content: bytes, metadata: dict, btfs_bridge, cursor, governance):
    """
    Stores a large document into BTFS, saving its CID into the local pgvector DB.
    """
    cid = btfs_bridge.store(content, encrypt=True)
    if cid:
        # Save to local DB
        cursor.execute("INSERT INTO documents (content, btfs_cid, metadata) VALUES (%s, %s, %s)",
                       (content, cid, json.dumps(metadata)))
        # Anchor on RBB Chain via governance
        governance.anchor_seal(cid, "document_store")
        return cid
    return None
