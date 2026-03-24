# Spike: Marketplace bid / proposal APIs (Upwork & Freelancer)

**Date:** 2026-03-24  
**Purpose:** Inform `MarketplaceBidPort`, `BidSubmit` job kind, and orchestration (C3–C5). No product code in this task.

## 1. Upwork — can the API submit proposals?

- **Public API (OAuth 2.0):** Upwork exposes REST APIs for jobs, profiles, and related resources. Proposal *submission* is tightly coupled to marketplace rules, account type (agency vs freelancer), and often requires the official web/mobile flows for compliance.
- **Documentation:** [Upwork API documentation](https://developers.upwork.com/) — review current “Job Application” / proposal-related scopes; scopes and availability change over time.
- **Assessment:** Automated proposal submission via a stable, documented public endpoint is **uncertain** for a generic integration: many teams use browser automation or official partner programs instead. Expect **OAuth**, **per-client app approval**, and **strict rate limits**. Verify whether your account type can apply to jobs programmatically before committing to a port.

## 2. Freelancer.com — bids via API?

- **Documentation:** [Freelancer API](https://www.freelancer.com/api/docs) — project and bid-related methods exist (e.g. placing bids is discussed in API docs; exact path names may vary by API version).
- **Assessment:** Bidding is **more plausibly** API-driven than on some competitors, but still requires **OAuth**, project visibility, and account eligibility. **Rate limits** apply per app and user.

## 3. Auth summary

| Platform    | Typical auth        | Notes                                      |
|------------|---------------------|--------------------------------------------|
| Upwork     | OAuth 2.0           | Developer app, redirect URIs, scopes       |
| Freelancer | OAuth 2.0         | API keys / OAuth per Freelancer docs       |

## 4. Rate limits and restrictions

- Both platforms enforce **per-app and per-user throttling**; burst + daily caps are common.
- **Terms of service:** Automated bidding may violate marketplace rules if it impersonates users or bypasses required disclosures.
- **Recommendation:** Treat any automation as **opt-in**, **logged**, and **approval-gated** (aligns with ADR-008).

## 5. Recommendation: stub trait now or wait?

1. **Do not** implement `MarketplaceBidPort` or `BidSubmit` until a chosen platform’s **current** docs are read end-to-end and a **pilot account** can run a successful *sandbox or low-risk* call.
2. **Now:** Keep human-in-the-loop proposal flow (generate + persist + approve) as implemented in harvest; use **manual** submission on the marketplace UI if APIs are blocked.
3. **After validation:** Introduce a narrow `MarketplaceBidPort` with one method (e.g. `submit_bid`) and a single adapter for the platform you proved out; add `BidSubmit` + approval in the job worker mirroring `ProposalDraft`.

### References (verify live pages before implementation)

- Upwork Developers: https://developers.upwork.com/  
- Freelancer API docs: https://www.freelancer.com/api/docs  
