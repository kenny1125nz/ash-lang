# Action Items & Mandates — 9 July 2026 Board Meeting

> Tracking all action items, deadlines, owners, and verification methods.  
> Prior action items from previous meetings were not specifically enumerated in these minutes (previous minutes were approved as circulated); any open items from prior meetings are superseded by the actions below.

---

## Matters Arising — First Review at Next Board Meeting

| # | Priority | Action | Owner | Deadline | Verification |
|---|----------|--------|-------|----------|-------------|
| **MA-1** | **Matters Arising** | **SOC 2 Type II scoping: select audit firm, secure budget ($80–120K estimated), schedule kickoff.** Marketing reallocation of $4K/month funds the scoping phase. | Marcus Thorne | **Within 30 days** (8 Aug 2026) | Audit firm name and kickoff date reported to board. Budget line item approved. |
| **MA-2** | **Matters Arising** | **Pricing page ships: 1 dev free/unlimited forever; Team $29/seat/mo (min 3 seats, $87/mo); Enterprise per-seat annual starting ~$70K/yr. Runway math visible on the page. Free tier irrevocably codified in GOVERNANCE.md. Procurement-quotable in 60 seconds with no “contact us” gate on self-serve tiers.** | Marcus Thorne | **Within 30 days** (8 Aug 2026) | Live pricing page verified; GOVENANCE.md updated; founder communication published same day. |
| **MA-3** | **Matters Arising** | **Markdown engine ships as primary product interface. DSL repositioned to internal compile target. Migration tool (`ash migrate --to-markdown`) ships alongside. First-workflow-in-5-minutes is the onboarding metric.** | Marcus Thorne | **30 Sep 2026** (hard deadline) | Markdown engine live as default onboarding experience; shipped migration tool; engineering allocation inverted by end of July. |

---

## Full Action Item Register

### Governance & Compliance

| # | Action | Owner | Deadline | Verification |
|---|--------|-------|----------|-------------|
| 1 | Community governance model published: CLA/DCO policy, maintainer nomination process, contributor pathway (first PR → maintainer), public roadmap. Shipped simultaneously with pricing page — governance first, then pricing. | Marcus Thorne | Within 30 days (8 Aug 2026) | Documents live in repo. Sarah Chen reviews and signs off. |
| 2 | Reach out personally to @aibuilder — understand root cause of departure (burnout? competitor? natural attrition?). Report findings to board. | Marcus Thorne | By end of week (13 Jul 2026) | Findings included in next board pack. |
| 3 | Offer paid maintainer stipend / formal role to @devopsjane. Structure as part-time contract or stipend program. Include in broader community contributor program (top 5 contributors funded from $4K/month marketing reallocation). | Marcus Thorne | Within 2 weeks (23 Jul 2026) | Signed agreement or offer letter. @devopsjane confirmation reported to board. |
| 4 | Personally reach out to top 10 contributors this quarter — understand what they need to stay. Report findings to board. | Marcus Thorne | This quarter (by next board meeting) | Summary of contributor sentiment in board pack. |
| 5 | Reach out to top 20 contributors individually before markdown engine ships — communicate format transition, explain rationale, solicit feedback, incorporate migration requirements. | Marcus Thorne | Before markdown engine ship (by 30 Sep 2026) | Contributor outreach log reported to board. |
| 6 | Quarterly contributor health check as standing board metric: contributor retention curves, sentiment surveys for top 20 contributors, first-PR-to-second-PR conversion, time-to-merge. | Marcus Thorne | Quarterly, starting next board meeting | Metrics in every board pack alongside financials. |
| 7 | Instrument contributor funnel: first PR → second PR → maintainer conversion rates. | Marcus Thorne | By next board meeting | Data presented in board pack. |
| 8 | Get open PRs under 5 and keep them there. Designate 2 engineers as official PR reviewers with SLAs. Implement triage rotation for 48-hour issue classification. | Marcus Thorne | By end of Q3 (30 Sep 2026) | Open PR count and response times in board pack. |
| 9 | Contributor format breakdown data: % PRs by format (.ash vs .md), % support tickets by format, % templates by format, format preference of top 20 / new / repeat contributors. | Marcus Thorne | By next board meeting | Data table in board pack. |
| 10 | Codify free tier as irrevocable in GOVERNANCE.md — "Individual developers receive a permanent, irrevocable, unlimited free tier. This commitment may not be modified, degraded, or revoked without a community governance vote." | Marcus Thorne | Within 30 days (8 Aug 2026) | GOVERNANCE.md updated; Sarah Chen verifies language. |

---

### Enterprise Pipeline & Revenue

| # | Action | Owner | Deadline | Verification |
|---|--------|-------|----------|-------------|
| 11 | Drop error rate below 2% by end of Q3; scope path to 0.5% (enterprise SLA target). Context compaction ships August; output schema validation added. Dedicated reliability rotation within engineering owns the metric. | Marcus Thorne | Sub-2% by 30 Sep 2026; path to 0.5% scoped by next board meeting | Error rate trend line in every board pack. |
| 12 | EU AI Act Article 14 audit trail compliance: confidence scoring on every agent output, decision rationale logged at every branch point, human-override events with identity/timestamp/justification. | Marcus Thorne | By Jan 2027 (enforcement date) | Audit trail sample submitted to board for review. |
| 13 | Detailed Article 14 scoping document with engineering estimates. | Marcus Thorne | By next board meeting | Document distributed in board pack. |
| 14 | RBAC design doc: creator, approver, operator role separation with separate audit trails. | Marcus Thorne | By next board meeting | Design doc circulated. |
| 15 | RBAC ships (Q4 engineering priority, after context compaction and markdown engine). | Marcus Thorne | Q4 2026 | RBAC functional in Ash Cloud. |
| 16 | Enterprise SSO roadmap: SAML 2.0 and OIDC with Azure AD and Okta. Auth foundation stabilised first. | Marcus Thorne | Roadmap by next board meeting; target Q4 2026 | Roadmap document; be transparent with 6 enterprise evaluations that SSO is Q4. |
| 17 | Multi-region architecture roadmap — scoped, costed, prioritized alongside SOC 2 and markdown engine. | Marcus Thorne | By next board meeting | Architecture document in board pack. |
| 18 | Data residency matrix: where Ash Cloud data lives, where it can be deployed (including EU regions), data sovereignty guarantees. | Marcus Thorne | By next board meeting | Matrix document in board pack. |
| 19 | Disaster recovery plan with defined RTO and RPO targets. | Marcus Thorne | Within 90 days (7 Oct 2026) | DR runbook documented; Elena Vasquez offered review. |
| 20 | Enterprise champion map for all 6 accounts: named champion, role, deployment status, internal influence, format dependency (can workflows be expressed in markdown today?). | Marcus Thorne | By next board meeting | Champion map in board pack. |
| 21 | Enterprise pipeline restatement: distinguish qualified opportunities (budget-holder engaged) from evaluation leads (technical team interested, no budget authority). | Marcus Thorne | By next board meeting | Restated pipeline in board pack. |
| 22 | Formal loss review on the bash-script deal: what was demoed, why they chose bash, findings reported to board. | Marcus Thorne | By next board meeting | Loss review document in board pack. |
| 23 | Security architecture whitepaper, compliance overview (SOC 2 roadmap, audit trail capabilities, RBAC roadmap), and RFx response template. | Marcus Thorne | By end of Q3 (30 Sep 2026) | Documents produced; GTM and engineering teams jointly deliver. |
| 24 | Security engineer JD drafted and sourcing begins (replaces AE hire; budget redirected from AE OTE of $120–150K). | Marcus Thorne | JD drafted this week (13 Jul 2026); hire within 60 days | JD approved; candidate pipeline in board pack. |
| 25 | VPC single-tenant deployment SKU scoping and timeline — prioritized as first enterprise deployment model. | Marcus Thorne | By next board meeting | Scoping document in board pack. |
| 26 | Startup program scoping (Elena's proposal): Ash-sponsored Team tier for pre-seed/seed companies, application-based, renewable annually. | Marcus Thorne | By next board meeting | Program design document in board pack. |
| 27 | Auditor-legibility validation: before presenting markdown engine to any enterprise compliance team, validate that a non-engineer can independently read, trace, and understand a 20-step markdown workflow. | Marcus Thorne | Before first enterprise compliance presentation | Validated with a non-technical reviewer; result reported to board. |
| 28 | Enterprise champion migration assessment: for FintechCo and DataCorp, document current workflows, format used, and whether any depend on DSL capabilities the markdown engine doesn't yet support. | Marcus Thorne | By next board meeting | Assessment in board pack alongside champion map. |

---

### Product & Engineering

| # | Action | Owner | Deadline | Verification |
|---|--------|-------|----------|-------------|
| 29 | Engineering allocation inverted: markdown engine and context compaction receive majority of resources. DSL drops to maintenance (bug fixes only, no new features, no syntax extensions). DSL allocation not to exceed 20% after inversion. | Marcus Thorne | By end of July 2026 (31 Jul) | Updated engineering sprint allocation reported to board. |
| 30 | Context compaction ships — addresses largest error-rate driver (truncated output at context limits). Reliable execution up to ~100 steps. | Marcus Thorne | August 2026 | Shipped; error rate trend validates impact. |
| 31 | Bounded execution depth with explicit halt-on-uncertainty behavior specced into the August context compaction release. | Marcus Thorne | August 2026 | Spec included in compaction release. |
| 32 | Ash Cloud 90-day reliability roadmap: error rate sub-2%, uptime 99.9%, no new cloud features until reliability targets met. | Marcus Thorne | End of Q3 (30 Sep 2026) | Metrics in board pack; feature freeze confirmed. |
| 33 | `ash discover` v2 with recommendation engine — "for this task, this agent has the highest community success rate." | Marcus Thorne | End of Q3 (30 Sep 2026) | Shipped and measurable via agent-selection support tickets. |
| 34 | Key-person risk on execution engine resolved: pair senior backend engineer (starting August) on engine internals. By 31 Dec 2026, at least one other person can modify execution graph and debug parser failures without Marcus. | Marcus Thorne | 31 Dec 2026 | Named engineer with demonstrated capability; verified by code review and incident response drill. |
| 35 | Markdown engine Q4 roadmap: close expressiveness gap — parallel dispatch, conditional branching, retry-with-fallback, state-passing — targeting 95% parity with DSL capabilities. | Marcus Thorne | Q4 roadmap by next board meeting; delivery Q4 2026–Q1 2027 | Roadmap document; progress reported quarterly. |
| 36 | Cost per active workflow by format (.ash vs .md) tracked and reported at every board meeting. | Marcus Thorne | Ongoing, starting next board meeting | Metric in board pack. |
| 37 | DSL sunset criteria tracked: (a) markdown expressiveness at 95% parity, (b) DSL active users <5% of WAU, (c) cost per .ash workflow >$200/mo. Sunset decision additionally gated on at least one enterprise contract with fully markdown-native deployed workflows. | Marcus Thorne | Ongoing, reviewed quarterly | Criteria dashboard in board pack. |
| 38 | Getting-started guide that answers top 5 questions from new users — shipping on page one. | Marcus Thorne | By end of month (31 Jul 2026) | Published guide; community feedback reviewed. |
| 39 | 20 workflow templates for common use cases (CI/CD, code review, deployment, testing, onboarding), each published as a runnable `.md` file in a public repo. Funded from $4K/month marketing reallocation. | Marcus Thorne | By end of Q3 (30 Sep 2026) | Templates published and linked from documentation. |
| 40 | Contributor migration tool: `ash migrate --to-markdown` ships alongside markdown engine. | Marcus Thorne | 30 Sep 2026 | Tool shipped and documented. |

---

### GTM, Marketing & Data

| # | Action | Owner | Deadline | Verification |
|---|--------|-------|----------|-------------|
| 41 | AE job description withdrawn. Replaced with solutions engineer hire (runs technical evaluations, builds champion relationships, generates enablement artifacts, no quota). | Marcus Thorne | AE JD pulled immediately; solutions engineer JD by next board meeting | AE posting removed; solutions engineer JD circulated. |
| 42 | $12K/month marketing reallocated: $4K to SOC 2 scoping, $4K to developer content & workflow templates, $4K to community stipend program (including @devopsjane). Zero paid brand awareness until revenue threshold meaningful. | Marcus Thorne | By end of month (31 Jul 2026) | Budget reallocation confirmed; stipend program launched. |
| 43 | Activation funnel instrumented: signup-to-first-execution cohorts, first-execution-to-second cohorts, team-invite conversion rates. Report activation within 48 hours and within week one. | Marcus Thorne | By next board meeting | Cohort data in board pack. |
| 44 | Churn decomposition by driver: activation churn, capability churn, competitive churn. Exit survey instrumented for departing users. | Marcus Thorne | By next board meeting | Churn analysis and survey results in board pack. |
| 45 | Net WAU growth broken out by cohort (new activation minus churned). L12M retention curve. All activation curves by cohort. | Marcus Thorne | By next board meeting | Cohort charts in board pack. |
| 46 | Execution distribution by org (top decile, top 5%, median). Power-user cohort analysis. | Marcus Thorne | By next board meeting | Distribution data in board pack. |
| 47 | Team-size distribution: how many orgs have 2+ users, 3+, etc. Correlation between team size and enterprise inquiry. | Marcus Thorne | By next board meeting | Data in board pack. |
| 48 | Behavioral segmentation: usage patterns of 336 non-paying users vs. 4 paying users (executions/month, team size, workflow complexity, agent count, time on platform). | Marcus Thorne | By next board meeting | Segmentation analysis in board pack. |
| 49 | Pricing model run against actual usage data: projected revenue under per-seat model, validation of power-law assumptions. Review 3-seat minimum against team-size distribution data. | Marcus Thorne | By next board meeting | Revenue projection and minimum-seat analysis in board pack. |
| 50 | Revenue forecast model: how many free teams produce one qualified enterprise lead; conversion rate from qualified lead to closed deal. | Marcus Thorne | By next board meeting | Model in board pack. |
| 51 | Monitor team-invite conversion rates, trial-to-paid conversion rates, and correlation between team size and enterprise inquiry — leading indicators of whether per-seat model is suppressing or accelerating the bottom-up motion. | Marcus Thorne | Ongoing, reported at next board meeting | Metrics in board pack. |

---

### Communication & Transparency

| # | Action | Owner | Deadline | Verification |
|---|--------|-------|----------|-------------|
| 52 | Founder communication post published alongside pricing page: Marcus's voice, undefended, explaining burn rate ($310K/mo), runway (14 months), free tier commitment, and monetization decision. Publish same day as pricing page. | Marcus Thorne | Within 30 days (8 Aug 2026) | Post live; Sarah Chen reviews tone and content. |
| 53 | Founder communication on licensing: MIT is permanent and irrevocable, board was unanimous, TaskWeaver's launch prompted the question but the answer is execution velocity — not license change. | Marcus Thorne | Within 30 days (8 Aug 2026) | Post live. |
| 54 | Community discussion thread opened in repo Discussions tab on pricing launch day. Marcus and engineering leads answer questions directly, publicly, in real time. | Marcus Thorne | Day of pricing launch | Thread live and active. |
| 55 | Runway math visible on the pricing page itself — not in a separate blog post. Short, honest: "Ash burns $310,000/month. We have 14 months. Here's what stays free forever, here's what costs money, and here's why." | Marcus Thorne | Within 30 days (8 Aug 2026) | Pricing page reviewed and confirmed by board. |

---

## Decisions Ratified by Resolution

These are formal resolutions already adopted by the board; they are recorded here for operational tracking:

| Resolution | Summary | Owner | Key Date |
|------------|---------|-------|----------|
| **R1 — Monetization** | Per-seat pricing: Free 1-dev unlimited forever; Team $29/seat/mo (min 3 seats, $87/mo); Enterprise per-seat annual ~$70K/yr. 3-seat minimum reviewed at next board meeting against team-size data. Metered model retained as fallback enterprise SKU. | Marcus Thorne | Pricing page ships within 30 days |
| **R2 — Licensing** | MIT stays permanently and irrevocably. Codified in GOVERNANCE.md. No license-change proposal before $10M ARR + 50 enterprise customers. Any future proposal requires community governance process (RFC, discussion, contributor vote). | Board unanimous | GOVERNANCE.md updated within 30 days |
| **R3 — DSL Investment** | Markdown-first product strategy. Markdown engine ships primary product interface by 30 Sep 2026. Engineering allocation inverts by 31 Jul 2026. DSL becomes compile target; sunset evaluated when 95% parity, <5% DSL WAU, >$200/workflow cost, and at least one enterprise contract on markdown-native workflows. | Marcus Thorne | Multiple milestones (see actions above) |

---

## Prior Action Items

Previous meeting minutes were approved as circulated. No prior action items were explicitly referenced as outstanding in these minutes. The board's review of any open items from the previous meeting is subsumed into the action items above. Should any prior action items remain unresolved, they are to be carried forward in the next board pack.

---

*End of action items. Next review: at the following board meeting (tentatively Q4 2026, date TBD). All "Matters Arising" items are first on the agenda.*
