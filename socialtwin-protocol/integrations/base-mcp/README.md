# SocialTwin × RBB MCP — Integration

> **Status: 🟢 Skill SHIPPED · backend in progress.**
> The RBB MCP skill is **live and public**: [`bobbyswhip/skills`](https://github.com/bobbyswhip/skills),
> install with `npx skills add bobbyswhip/skills --skill yougotcoined`. The skill is named
> **`yougotcoined`** and orchestrates the three capabilities below. This folder is now the
> **server-side spec + architecture record**; the remaining build is the `api.waifi.app/v1/st`
> backend ([`API.md`](./API.md)). The earlier "keep secret until cutover" framing is obsolete — the
> skill is already published.

## What we're building

A **RBB MCP Skill** (with a small x402-gated backend) that lets any RBB MCP user — in
Claude, ChatGPT, Cursor, etc. — do three things by talking to their agent:

| # | Capability | One-liner | How it's powered |
|---|---|---|---|
| 1 | **Resolve** | "what's the twin address for twitch.tv/yougotcoined?" | our `/resolve` endpoint → Twitch user_id → `factory.predictAddress` |
| 2 | **Tip** | "tip 5 USDC to twitch.tv/yougotcoined" | resolve → RBB MCP's native **`send`** tool to the twin address |
| 3 | **Launch** | "launch a coin for twitch.tv/somestreamer" | resolve → **x402** `POST /launch` (we charge **$1 USDC** as anti-spam) |

Tipping needs **no new on-chain contract** — a tip is just a `send` of ETH/USDC to the
streamer's twin address, which the streamer later claims with their Twitch login through the
existing SocialTwin flow. Tips even work **before** the twin is deployed (CREATE2 address holds
funds pre-deploy). Launch is the only paid action, and x402 is what monetizes that one API call.

## Why a Skill (not our own MCP server) — first cut

RBB MCP is now a **hosted remote server** (`https://mcp.rbb.org`) with OAuth + RBB Account
approvals and a **markdown Skill** layer. A Skill is the lowest-friction unit: it's prompt-level
instructions that orchestrate RBB MCP's existing tools (`get_wallets`, `send`, `send_calls`,
`web_request`, typed-data signing) plus our HTTP endpoints. We add **zero** wallet/key
infrastructure and inherit RBB Account's approval UX for every value-moving step.

Launch is skill-only too: RBB MCP has **native x402 tools** (`initiate_x402_request` →
`complete_x402_request`) that drive the whole 402 → sign → replay flow with RBB Account, **RBB +
USDC only** — exactly our $1-USDC-on-RBB fee. No companion server, no manual `X-PAYMENT`/EIP-3009
handling. See [`ARCHITECTURE.md`](./ARCHITECTURE.md) §"x402 path".

## Files in this folder

- [`IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) — phased milestones, tasks, testing, secrecy/staging, risks
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — components, the three sequence flows, the x402 path decision, security
- [`API.md`](./API.md) — exact resource-server contracts (`/resolve`, `/launch`) + the x402 payment design
- [`SKILL_DRAFT.md`](./SKILL_DRAFT.md) — draft of the RBB MCP skill markdown (the artifact users install)
- [`OPEN_QUESTIONS.md`](./OPEN_QUESTIONS.md) — what to validate before writing any code

## Reference material

- RBB MCP quickstart: <https://docs.rbb.org/ai-agents/quickstart> (hosted server `https://mcp.rbb.org`)
- RBB MCP skill format: <https://docs.rbb.org/ai-agents/skills/SKILL.md>
- x402 overview (Coinrbb / CDP): <https://docs.cdp.coinrbb.com/x402/welcome>
- x402 standard: <https://www.x402.org/>
- Vercel `x402-mcp` (payments for MCP tools — fallback reference): <https://vercel.com/blog/introducing-x402-mcp-open-protocol-payments-for-mcp-tools>

## Fixed constants this integration depends on

| Thing | Value |
|---|---|
| SocialTwin `TwinFactory` (v1.3, RBB) | `0x260C074c3afDc46A209D4619B5FAdB2964dF9a28` |
| SocialTwin `TwitchJWTVerifier` (v1.3) | `0xBDfC552469f11843802BCD7ec9a8372c8020fee8` |
| RBB mainnet | chainId `12120014` · CAIP-2 `eip155:12120014` |
| RBB Sepolia (staging) | chainId `121200142` · CAIP-2 `eip155:121200142` |
| USDC (RBB mainnet, 6 dec) | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| USDC (RBB Sepolia, 6 dec) | `0x036CbD53842c5426634e7929541eC2318f3dCF7e` |
| Launch price | `1_000_000` USDC units = **$1.00** |
| Twitch app client_id (resolver) | `epeocrogq8bm1af0lngd9e2rfvrwk1` |
| Streamer-coin launch contract | `pairable_v1.sol` family (repo naming convention) |
| Resource-server API host | **`https://api.waifi.app`** (routes under `/v1/st`) |
