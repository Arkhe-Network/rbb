with open("orchestrator/src/rl/debate_consensus_reward.rs", "r") as f:
    content = f.read()
content = content.replace("use std::sync::Arc;", "")
with open("orchestrator/src/rl/debate_consensus_reward.rs", "w") as f:
    f.write(content)
