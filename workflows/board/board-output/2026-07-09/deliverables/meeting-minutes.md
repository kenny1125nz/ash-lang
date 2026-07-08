# MINUTES OF A MEETING OF THE BOARD OF DIRECTORS

## of Ash Technologies, Inc.

**Date:** 9 July 2026
**Time:** 09:00 (local time)
**Location:** By video conference

---

### 1. ATTENDANCE

The following directors were present throughout the meeting, constituting a quorum of the Board:

- **Marcus Thorne** — Executive Director, Co-Founder & Chief Executive Officer
- **Priya Nair** — Investor Director, Partner at Framework Capital
- **Sarah Chen** — Director, Open-Source & Community
- **James Okonkwo** — Independent Director, DevTools Go-To-Market
- **Dr. Elena Vasquez** — Independent Director, Enterprise Architecture

---

### 2. DECLARATIONS OF INTERESTS

No conflicts of interest were declared by any director in relation to the business to be transacted at the meeting.

---

### 3. ADMINISTRATIVE

**3.1 Previous Minutes**

The minutes of the previous board meeting were reviewed and approved as circulated.

**3.2 Agenda**

The agenda for the meeting was adopted as circulated.

---

### 4. CEO BUSINESS UPDATE

Marcus Thorne presented the CEO business update covering the following matters:

**4.1 Key Metrics**

The company reported 5,200 monthly active workflows (up from 1,400 in the prior quarter, representing 22% month-over-month growth), across 1,500 unique organizations (up from 600), with average workflows per organization rising from 2.3 to 3.5. Weekly active users stood at 340 (up from 180), with monthly churn declining from 18% to 12%. Net Promoter Score moved from 28 to 32. Day-30 retention improved from 16% to 22%.

GitHub stars doubled to 8,200. Unique contributors doubled to 94, with repeat contributors more than doubling from 6 to 14.

Ash Cloud was live with 340 free users and 4 paying customers, producing $400 in monthly recurring revenue. Error rate was 3.2% of executions. Uptime was 99.7%, with one 45-minute outage during an auth migration recorded as the sole incident.

**4.2 Financials**

Cash on hand: $4,400,000 (down from $5,400,000). Monthly burn averaged $310,000 (up from $275,000). Runway: 14 months (down from 19). Burn composition: $240,000 personnel, $28,000 infrastructure, $22,000 tools and services, $12,000 marketing, $8,000 legal. Headcount: 18, with plans to add 3.

Revenue: $400 per month. Enterprise pipeline comprised six accounts with an estimated aggregate ACV of $600,000. None had cleared security review. FintechCo was identified as the closest to close, at 30% probability, targeting Q4 2026.

**4.3 Achievements**

The CEO reported the successful launch of `ash discover` (zero-config onboarding), community growth including a key repeat contributor (@devopsjane, 9 pull requests in the quarter), a positive customer testimonial from a fintech startup, the EU AI Act enforcement date of January 2027 as a regulatory tailwind for the company's governance positioning, and zero staff departures during the quarter.

**4.4 Challenges**

The CEO reported: Ash Cloud shipped six weeks late due to auth integration issues; a deal was lost to a bash script, with the prospect citing the `.ash` DSL learning curve as a barrier; context window limits impairing workflows exceeding approximately 50 steps; agent selection confusion among users; key-person risk (the CEO being the sole individual with deep understanding of the parser and execution engine); and the enterprise pipeline remaining entirely pre-revenue, with all six evaluations stalled at security review.

**4.5 Market Developments**

The CEO noted: the emergence of TaskWeaver (a YC W26, 3-person competitor launching with a `.md`-based task runner, 1,200 GitHub stars in two weeks); GitHub Copilot's agent workflow mode in private beta; MCP adoption across 15 agent tools; and four YC-batch companies entering the AI workflow/agent orchestration category.

**4.6 CEO's Asks of the Board**

The CEO requested board guidance on four matters: (a) monetization model clarity; (b) licensing strategy; (c) the future of the `.ash` DSL relative to the markdown engine; and (d) SOC 2 prioritisation and budget.

**4.7 Questions from the Board**

Extended question-and-answer sessions were conducted by each director:

- **Priya Nair** questioned the CEO on revenue forecasting, the AE hire timing, SOC 2 timeline compression, free-to-paid unit economics, net WAU growth and retention, DSL resource allocation, TaskWeaver velocity, and marketing spend attribution.

- **Sarah Chen** questioned the CEO on the silence of a former top contributor (@aibuilder), the open PR and issue backlog, the 15% repeat contributor rate, the status of @devopsjane (unpaid community labour), documentation gaps and agent selection confusion, the DSL from a community contribution perspective, licensing and community trust, monetization's impact on community trust, and the absence of community governance infrastructure (CLA/DCO, maintainer pathways, contributor roadmap).

- **Dr. Elena Vasquez** questioned the CEO on SOC 2 timing and cost, the 3.2% error rate as an enterprise blocker, EU AI Act Article 14 audit trail compliance, single-region deployment and DR planning, the absence of a dedicated security hire, on-prem and VPC deployment SKUs, RBAC and role separation, the DSL as a procurement rejection criterion, context window limits and deterministic execution, auth instability and enterprise SSO, the impact of a license change on enterprise procurement, procurement-compatible pricing, and enterprise pipeline probability-weighted mathematics.

- **James Okonkwo** questioned the CEO on the invisible activation funnel, the 1.2% conversion rate as a signal rather than an early-stage artifact, the metered pricing model, team invite conversion and multi-user org metrics, the absence of internal champions in the enterprise pipeline, the AE hire timing and risk, marketing spend allocation, TaskWeaver as a GTM challenge, the markdown engine timeline, monetization execution urgency, power-user cohort analysis, the loss to a bash script as a GTM failure requiring formal review, churn decomposition, and the Ash Cloud reliability recovery plan.

The CEO responded to all questions, acknowledging data gaps (activation cohorts, churn decomposition, execution volume distributions, contributor retention curves, champion maps), accepting the board's direction on: withdrawing the AE hire in favour of a solutions engineer; withdrawing the licensing question and recommitting to MIT; demoting the DSL to an internal compile target with markdown as the primary product interface by end of Q3; reallocating the $12,000 marketing budget ($4,000 to SOC 2 scoping, $4,000 to developer content and templates, $4,000 to community stipends); and committing to a permanent free tier for individual developers.

---

### 5. STRATEGIC DISCUSSION — MONETIZATION

The board engaged in a detailed discussion on the company's monetization model. Positions were advanced by all five directors.

**5.1 Points of Agreement**

All directors agreed on: (a) a permanent, irrevocable free tier for individual developers as the top of the adoption funnel; (b) enterprise contracts as the only revenue path that closes the runway mathematics; (c) a pricing page to ship within 30 days, transparent and quotable by a procurement analyst in 60 seconds; (d) the free-tier commitment to be codified in GOVERNANCE.md as irrevocable; (e) community governance to ship alongside pricing, not after it; (f) SOC 2 Type II scoping to begin within 30 days; (g) the AE hire to be withdrawn and replaced with a solutions engineer; (h) the $12,000 marketing budget to be reallocated as described above; (i) the markdown engine to ship as the primary product interface by end of Q3 2026; (j) MIT license to remain, with no BSL; and (k) the model to be reviewed against actual usage data at the next board meeting.

**5.2 Points of Disagreement**

The board divided 3–2 on the core pricing structure. The per-seat position (Priya Nair, Sarah Chen, Dr. Elena Vasquez) proposed: Free — one developer, unlimited workflows, full product, forever; Team — $29/seat/month, minimum 3 seats ($87/month), 14-day trial triggered by second developer joining; Enterprise — per-seat annual, starting at 200 seats (~$70,000/year). The metered position (James Okonkwo, and the CEO's opening) proposed: Free — 100 workflow executions/month per org; Pro — $0.10/execution above 100, with volume discounts; Enterprise — annual committed-volume contracts. The board additionally discussed the free tier definition (unlimited, no meter versus 100 executions/month cap) and the team-tier minimum (the board settled at 3 seats rather than the originally proposed 5, following discussion of the risk to 2-person founder teams).

**5.3 Resolution**

The following resolution was adopted by a 3–2 majority (Priya Nair, Sarah Chen, and Dr. Elena Vasquez in favour; James Okonkwo dissenting; the Chair, Marcus Thorne, casting the deciding vote with the majority):

> RESOLVED THAT the board adopts a per-seat pricing model as the primary monetization structure for Ash, structured as follows:
>
> (i) **Free tier** — one developer, unlimited workflow executions, full product, forever, irrevocable and codified in the repository's GOVERNANCE.md;
>
> (ii) **Team tier** — $29 per seat per month, minimum 3 seats ($87/month), with a 14-day trial triggered automatically when a second developer joins an organization, including shared workflow dashboards, team audit trails, and CI integration;
>
> (iii) **Enterprise tier** — per-seat annual contracts starting at 200 seats (~$70,000/year), volume discounts above 500 seats, including SSO, RBAC, VPC single-tenant deployment, EU AI Act Article 14 audit trail exports, and priority support, with a published per-seat rate and a self-serve pricing page that a procurement analyst can quote in 60 seconds without contacting sales.
>
> The board further resolves that:
>
> (a) the pricing page, the community governance model (CLA/DCO, maintainer nomination process, contributor pathway, public roadmap), and a founder communication from the CEO explaining the runway mathematics and the decision shall ship together within 30 days;
>
> (b) the 3-seat team-tier minimum shall be reviewed at the next board meeting against team-size distribution data, with explicit evaluation of whether the minimum suppresses champion formation among 2–4 person teams;
>
> (c) the startup program proposal — Ash-sponsored team tier for pre-seed and seed-stage companies, application-based, renewable annually — shall be scoped and presented at the next board meeting; and
>
> (d) the board retains the metered enterprise committed-volume model as a fallback enterprise SKU structure should execution-distribution data at the next board meeting demonstrate that a per-seat model is materially suppressing adoption.

The Chair recorded that his vote with the majority was cast after being persuaded by the procurement and trust arguments of the majority. James Okonkwo's full dissenting argument was recorded in the meeting record.

---

### 6. STRATEGIC DISCUSSION — LICENSING

The board conducted a comprehensive review of the company's licensing position. All five directors presented positions.

**6.1 Points of Agreement**

All five directors agreed that: (a) the MIT license shall remain, permanently and irrevocably; (b) a license change at the current stage would be catastrophic — the combined cost of legal fees, community trust erosion, enterprise pipeline destruction, and GTM engine damage having been independently priced by four directors as exceeding the company's current financial capacity; (c) the licensing question was a category error — TaskWeaver did not fork Ash's code but copied the `.md` format, which was already MIT-licensed, and the company's engineering strategy had already designated the markdown engine (not the DSL execution engine) as the product; (d) the real competitive moat is not a legal clause but a combination of community ownership, enterprise governance features, adoption depth, format entrenchment, and execution velocity; (e) TaskWeaver's existence constitutes market validation, not a licensing threat; and (f) the HashiCorp BSL relicense serves as the cautionary precedent — a company with over $500 million in annual revenue suffered permanent community damage from a license change, which Ash at $400 MRR could not survive.

**6.2 Conditions Precedent to the Moat Holding Under MIT**

The board identified five conditions that must be met for the competitive moat to hold under MIT: (i) the markdown engine must ship as the primary product interface and the adoption gap with TaskWeaver must close; (ii) community governance and contributor ownership must be demonstrably real, with a contributor retention rate above 25% within two quarters; (iii) enterprise governance features — SOC 2, RBAC, Article 14 audit trails — must ship before the cash runs out; (iv) adoption depth must compound faster than churn erodes it; and (v) the `.md` format must achieve sufficient community and industry adoption to become the default standard before a proprietary alternative achieves equivalent or greater distribution.

**6.3 Resolution**

The following resolution was adopted unanimously:

> RESOLVED THAT the board unanimously affirms that Ash remains MIT-licensed, permanently and irrevocably, and that this commitment shall be codified in the repository's GOVERNANCE.md as a binding project covenant — not a pricing-page promise, not a board-meeting resolution subject to quarterly review, but a constitutional provision of the project's governance model that may not be modified without a community governance process including an RFC, a public discussion period, and a contributor vote.
>
> The board further resolves that:
>
> (i) No license-change proposal shall be entertained by this board before Ash reaches $10,000,000 ARR and maintains at least 50 enterprise customers with annual contracts — the combined threshold below which the company lacks the financial resilience and customer concentration to survive the community fracture and enterprise pipeline destruction that a license change would trigger. Even above that threshold, any proposal must satisfy the community governance process specified above, on the principle that the license is not solely a corporate asset but is the covenant under which 94 contributors built the project and 1,500 organizations adopted it.
>
> (ii) The defense against competitive absorption under MIT is defined as the following five execution items, which the CEO has committed to and which the board will track at every meeting until completion: (a) the markdown engine ships as the primary product interface by end of Q3 2026, closing the product gap that TaskWeaver exploited; (b) SOC 2 Type II scoping begins within 30 days, audit firm selected, kickoff scheduled; (c) error rate drops below 2% by end of Q3, with a scoped path to 0.5%, driven by context compaction (August 2026) and output schema validation; (d) EU AI Act Article 14 audit trail compliance ships by January 2027 — confidence scoring, decision rationale logging, and human-override audit events; and (e) community governance model — CLA/DCO policy, maintainer nomination process, contributor pathway from first PR to maintainer, public roadmap — is published before or simultaneously with the pricing page within 30 days.
>
> (iii) The board recognizes that the format — the `.md` file structure for expressing AI orchestration — is the company's primary strategic asset, not the execution engine, and that this format is already open and MIT-licensed. The company's competitive strategy must therefore rest on being the best implementation of the open format (velocity, reliability, enterprise governance), not on controlling access to the format through legal means. Future competitive threats shall be evaluated on this basis.
>
> (iv) The board directs the CEO to publish, alongside the pricing page within 30 days, a founder communication that addresses the licensing question transparently — stating that MIT is permanent and irrevocable, explaining why the board considered and rejected any alternative, and making the affirmative case that Ash's moat is adoption depth, community trust, enterprise governance, and execution velocity, not a license clause.
>
> (v) The board records James Okonkwo's observation that the per-seat monetization model and the MIT license form a single trust posture, and that the combined effect of per-seat friction at the team-formation threshold and any perceived license instability would compound to suppress the bottom-up adoption flywheel. The board's rejection of any license change mitigates the compound risk. The per-seat friction remains, and the board will review its effect on team-invite conversion rates at the next meeting.

---

### 7. FUNCTIONAL DEEP DIVE — DSL VERSUS MARKDOWN ENGINE

The board conducted a detailed review of the `.ash` DSL and the markdown engine, including the engineering resource allocation between the two.

**7.1 Points of Agreement**

All directors agreed that: (a) the markdown engine must ship as the primary product interface by September 30, 2026; (b) the `.ash` DSL is to be repositioned as an internal compile target and documented as advanced usage, not the primary onboarding path; (c) the engineering resource allocation must invert from the current 60%/30%/10% split (DSL parser/Ash Cloud/markdown prototype) with the markdown engine and context compaction receiving the majority of engineering resources; (d) the `.md` format — not the `.ash` syntax — is the company's primary strategic asset; (e) the DSL is a genuine technical achievement whose expressiveness, determinism, and compactness are real capabilities; (f) the markdown engine must close the expressiveness gap before the DSL can be considered for sunset; (g) key-person risk on the execution engine must be resolved by end of Q4 2026; and (h) contributor format breakdown data (`.ash` versus `.md` pull requests) must be collected and reported.

**7.2 Resolution**

The following resolution was adopted unanimously:

> RESOLVED THAT the board adopts a markdown-first product strategy with the `.ash` DSL repositioned as an internal compile target and power-user feature, according to the following terms:
>
> (i) The markdown engine ships as the primary product interface by September 30, 2026 — the documented default, the tutorial path, the community contribution surface, and the first experience a new user encounters. This date is a hard deadline. The CEO shall report progress against it at every board meeting until it ships.
>
> (ii) Engineering allocation inverts by end of July 2026. The markdown engine and context compaction receive the majority of engineering resources. The `.ash` DSL and execution engine drop to maintenance — bug fixes, critical patches, and support for existing power-user workflows, but no new features, no syntax extensions, and no investment in parser expressiveness beyond what is required to keep the compile target functioning. The board authorizes the CEO to determine the precise allocation ratio based on team capacity, provided the DSL allocation does not exceed 20% of engineering resources after the inversion.
>
> (iii) The DSL continues as a user-facing format for power users whose workflows require expressiveness the markdown engine cannot yet provide (parallel dispatch, conditional branching, retry-with-fallback, state-passing), documented as advanced usage, not primary onboarding. The DSL's continued availability as a user-facing format is contingent on the markdown engine receiving priority engineering investment to close the expressiveness gap. The markdown engine's Q4 2026 roadmap shall target expressiveness parity for 95% of workflows currently executed on the platform.
>
> (iv) The board will evaluate sunset of the DSL as a user-facing format when the following three conditions are all met: (a) the markdown engine supports expressiveness equivalent to the DSL for 95% of workflows executed on the platform, including conditional branching, parallel dispatch, and retry-with-fallback; (b) the DSL's active user base drops below 5% of weekly active users; and (c) the cost per `.ash` workflow exceeds $200 per month. The sunset evaluation shall also consider whether any enterprise contract currently depends on DSL capabilities not yet replicated in markdown.
>
> (v) A migration tool (`ash migrate --to-markdown`) shall ship alongside the markdown engine.
>
> (vi) The CEO shall reach out personally to the top 20 contributors before the markdown engine ships to communicate the format transition, explain the rationale, solicit feedback, and incorporate migration requirements.
>
> (vii) The CEO shall validate, before presenting the markdown engine to any enterprise compliance team, that a non-engineer can independently read, trace, and understand the decision logic in a 20-step markdown workflow.
>
> (viii) The transition shall proceed in two phases: Phase one (July–August 2026) — markdown engine and context compaction receive priority, DSL drops to maintenance. Phase two (September–December 2026) — once the markdown engine ships, the engineering team scopes the expressiveness gap and builds the Q4 markdown roadmap to close it. The DSL continues at maintenance throughout.
>
> If the markdown engine slips past September 30, 2026, the board shall reconvene to evaluate whether the DSL's continued maintenance allocation is sustainable against the competitive timeline TaskWeaver has established.

---

### 8. CLOSED SESSION (WITHOUT CEO)

The independent and investor directors — Priya Nair, Sarah Chen, James Okonkwo, and Dr. Elena Vasquez — convened in closed session, Marcus Thorne having been excused.

The directors assessed CEO performance, board composition, governance quality, founder-investor alignment, and key risks not fully addressed in the open session.

**8.1 CEO Performance**

The directors unanimously agreed that Marcus Thorne is the right person to lead the company at this stage, but identified five critical gaps requiring attention: (a) he is still operating as lead engineer rather than CEO and must invert his time allocation; (b) he lacks a credible operating rhythm, as evidenced by missing data instrumentation; (c) key-person risk is structural and extends across engineering, enterprise sales, and community relations; (d) he is not yet the enterprise CEO the pipeline requires; and (e) he has not fully separated emotional attachment to the DSL from the strategic needs of the business. The board does not recommend a CEO change but directed specific corrective actions, including an immediate engineering reallocation, hiring an operational partner within six months, structured enterprise coaching from Elena Vasquez and James Okonkwo, and closure of all identified data gaps by the next board meeting.

**8.2 Board Composition**

The board reviewed its composition and identified two blind spots requiring attention: (a) the absence of open-source IP and enterprise contracting legal expertise (the board recommended retaining external open-source legal counsel within 60 days); and (b) the absence of operational scaling experience (the board recommended evaluating the addition of an operating director at the next financing round). The board unanimously agreed that no changes to board composition were required at the present time beyond retaining external legal counsel.

**8.3 Governance Quality**

The board assessed meeting cadence (quarterly, appropriate), preparation quality (inadequate — the CEO did not present instrumented metrics on activation, churn, execution distribution, contributor retention, or champion mapping), and decision quality (high). The board directed Priya Nair to work with the CEO to define a mandatory standardized board pack template within 30 days, with detailed minimum contents specified, and recorded that inadequate preparation at the next meeting would trigger discussion of whether the CEO requires a Chief of Staff.

**8.4 Founder-Investor Alignment**

The board found overall alignment strong but identified three tensions requiring ongoing management: adoption velocity versus monetization urgency; the DSL as founder identity versus business liability; and fear of competitive absorption versus the MIT covenant. The board committed to managing these tensions through data-driven review at future meetings.

**8.5 Composite Risk Assessment**

The board identified the single largest risk as: the company runs out of money before any enterprise deal closes, not primarily because the pipeline is insufficient, but because the CEO has not yet made the transition from engineer to executive and the product is not yet enterprise-ready. The board directed Priya Nair to maintain a runway-risk dashboard for every meeting. Secondary risks identified included key-person dependency as an accelerant and competitive window closure (TaskWeaver's trajectory). The board noted that the October 2026 meeting will serve as the first formal checkpoint on the company's trajectory.

---

### 9. RESOLUTIONS

The following formal resolutions were adopted during the meeting:

**Resolution 1 — Monetization Model**

RESOLVED THAT the board adopts a per-seat pricing model as the primary monetization structure for Ash, with a Free Tier (one developer, unlimited workflow executions, full product, forever, irrevocable and codified in GOVERNANCE.md), a Team Tier ($29/seat/month, minimum 3 seats/$87 per month, 14-day trial triggered by second developer joining), and an Enterprise Tier (per-seat annual contracts starting at 200 seats/~$70,000/year, volume discounts above 500 seats, including SSO, RBAC, VPC single-tenant deployment, EU AI Act Article 14 audit trail exports, and priority support), with the pricing page, community governance model, and founder communication to ship within 30 days. Adopted 3–2.

**Resolution 2 — Licensing**

RESOLVED THAT the board unanimously affirms that Ash remains MIT-licensed, permanently and irrevocably, codified in GOVERNANCE.md as a binding project covenant not modifiable without community governance process; that no license-change proposal shall be entertained before Ash reaches $10,000,000 ARR and at least 50 enterprise customers with annual contracts, and thereafter only with community governance approval; and that the company's defense against competitive absorption rests on five specified execution items (markdown engine, SOC 2, error rate reduction, Article 14 audit trails, community governance model). Adopted unanimously.

**Resolution 3 — DSL and Markdown Engine**

RESOLVED THAT the board adopts a markdown-first product strategy with the `.ash` DSL repositioned as an internal compile target; that the markdown engine ships as the primary product interface by September 30, 2026; that engineering allocation inverts by end of July 2026 with the DSL allocation not to exceed 20% of engineering resources; that DSL sunset evaluation shall occur when markdown expressiveness reaches 95% parity, DSL active users fall below 5% of WAU, and cost per `.ash` workflow exceeds $200/month, subject to enterprise contract continuity; and that the transition proceeds in two phases with the markdown engine and context compaction receiving priority. Adopted unanimously.

---

### 10. CLOSING

There being no further business, the meeting was declared closed.

---

**SIGNED:**

_________________________
**Marcus Thorne**
Executive Director, Co-Founder & Chief Executive Officer
Chair of the Board

**Date:** _________________________
