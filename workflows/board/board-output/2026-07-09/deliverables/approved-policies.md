# Approved Policies and Reports

**Ash Board of Directors**
**Date of Approval:** 9 July 2026
**Effective Date:** Immediately upon adoption

These policies are adopted by formal resolution of the board and constitute the governing guidelines for the organization. They are binding on management and shall be reviewed quarterly.

---

## 1. AI Governance Framework

### 1.1 Mandate

Ash's mission is to eliminate enterprise liability by transforming unpredictable, stochastic AI behavior into auditable, deterministic business processes. This framework codifies the governance standards required to deliver that mission to regulated enterprises.

### 1.2 Auditability and Compliance

#### 1.2.1 EU AI Act Article 14 Compliance

The audit trail must satisfy Article 14 of the EU AI Act (enforcement date: 1 January 2027) by providing an evidentiary payload — not merely an execution trace — for every automated decision. The following must be logged for every agent invocation in a workflow:

- The model used, including provider and version identifier.
- The full prompt sent to the agent.
- The complete raw output received from the agent.
- A confidence or uncertainty score for each agent output, providing an auditor with a quantitative signal of reliability.
- The decision rationale at every branching point: the rule, threshold, or evaluation that selected a given branch, traceable to the specific agent output that triggered it.
- Human-override events, recorded with operator identity, timestamp, justification, and the preceding and resulting workflow state.

The audit trail must be exportable in a structured format (JSON or equivalent) that an independent auditor can ingest without access to the Ash platform.

**Accountability:** Engineering must ship Article 14 compliance by 1 January 2027. The CEO shall present a detailed scoping document with engineering estimates at the October 2026 board meeting.

#### 1.2.2 Auditor Legibility

Before the markdown engine is presented to any enterprise compliance team, the organization must validate that a non-engineer can independently read, trace, and understand the decision logic in a 20-step markdown workflow. Auditor-legibility is a demonstrated property of the format — not an assumed one.

**Accountability:** CEO to certify auditor-legibility validation before any enterprise compliance presentation. Dr. Elena Vasquez to review the validation output prior to enterprise use.

### 1.3 Deterministic Execution and State Control

#### 1.3.1 Error Rate Targets

The organization's error rate targets for production workflow execution are:

- **End of Q3 2026:** Below 2.0% — driven initially by context compaction (targeting August 2026) and output schema validation.
- **Path to enterprise-ready:** 0.5% with defined failure modes — the maximum acceptable rate for regulated production systems. A scoped engineering roadmap to 0.5% must be presented at the October 2026 board meeting.

#### 1.3.2 Deterministic Failure Detection

The execution engine must distinguish between deterministic failures (errors the engine detects and surfaces — malformed agent output, timeouts, state persistence failures) and stochastic failures (plausible but incorrect outputs the engine does not recognize as errors). The engine shall:

- Add output schema validation to every agent step so format errors are detected deterministically.
- Implement halt-on-uncertainty behavior: when output confidence drops below a defined threshold, or when context window saturation degrades quality, the engine must log a degradation event and halt gracefully with an operator-actionable error — not produce a confused partial output.
- Never silently degrade. Any execution reaching a boundary condition (context window limit, confidence collapse, unhandled agent output) must produce an explicit, logged failure.

**Accountability:** Context compaction and halt-on-uncertainty behavior shall be scoped into the August 2026 release. A dedicated reliability rotation within the engineering team shall own the error rate metric and report it in every board pack.

### 1.4 Enterprise Security and Deployment Posture

#### 1.4.1 SOC 2 Type II

SOC 2 Type II certification is a prerequisite for enterprise revenue. The organization shall:

- Select an audit firm and commence scoping within 30 days (by 9 August 2026).
- Allocate a budget of $80,000–$120,000 for the full Type II engagement.
- Engage a dedicated compliance resource — either a fractional hire or a new full-time security/compliance engineer — within 60 days. The CTO shall not be the sole security officer.
- Target certification by Q1–Q2 2027.

**Accountability:** CEO to report audit firm selection and kickoff date at the October board meeting.

#### 1.4.2 Role-Based Access Control

RBAC with role separation shall ship in Q4 2026. The minimum viable roles are:

- **Creator:** Authorized to define and modify workflows.
- **Approver:** Authorized to approve workflows for production execution. No individual may both author and approve the same workflow.
- **Operator:** Authorized to execute approved workflows and respond to halt events.

Each role action shall produce a separate, attributable audit log entry. This separation of duties is required for SOC 2 change management criteria.

**Accountability:** Engineering to present a detailed RBAC design document at the October board meeting. RBAC ships Q4 2026.

#### 1.4.3 Multi-Region Architecture and Disaster Recovery

Ash Cloud's current single-region deployment (us-east-1) is insufficient for regulated enterprise customers. The organization shall:

- Produce a multi-region architecture roadmap, scoped and costed, by the October board meeting.
- Deliver a data residency matrix showing where Ash Cloud data lives, where it can be deployed (including EU regions), and what data sovereignty guarantees can be made.
- Document a disaster recovery plan with defined Recovery Time Objective (RTO) and Recovery Point Objective (RPO) targets within 90 days.

**Accountability:** CEO to present the roadmap and residency matrix at the October board meeting. Dr. Elena Vasquez to review the DR architecture document prior to broad presentation.

#### 1.4.4 Enterprise SSO

The organization shall support SAML 2.0 and OpenID Connect with Azure AD and Okta. SSO shall ship in Q4 2026, following stabilization of the Auth0-based authentication infrastructure.

#### 1.4.5 VPC and Deployment SKUs

A VPC single-tenant deployment SKU is required to close enterprise deals. Engineering estimate: 4–6 months from commencement. The organization shall prioritize VPC single-tenant as the first enterprise deployment model. On-premises air-gapped deployment is deferred until a later stage.

#### 1.4.6 Dedicated Security Personnel

The board has approved redirecting the Account Executive budget toward a dedicated security/compliance engineer. This hire shall own SOC 2 evidence collection, penetration test coordination, enterprise security questionnaire responses, and security architecture. This is the hire that unlocks enterprise revenue.

### 1.5 Format and Procurement Compatibility

The markdown engine shall ship as the primary product interface by 30 September 2026. The `.ash` DSL shall be repositioned as an internal compile target and a documented power-user feature, not the default onboarding path. This decision is binding on the following grounds:

- The DSL introduces a custom programming language into regulated production stacks — a known procurement rejection criterion at Fortune 500 institutions.
- The markdown format is vendor-neutral, ecosystem-maintained, and auditor-legible without vendor-specific training.
- The format survives the vendor relationship — a procurement requirement the DSL cannot meet.

The board shall evaluate sunset of the DSL as a user-facing format when (a) the markdown engine supports expressiveness parity for 95% of platform workflows, (b) active DSL users represent less than 5% of weekly active users, and (c) the fully-loaded engineering cost per active `.ash` workflow exceeds $200 per month. Sunset is further conditioned on confirmation that no active enterprise contract depends on a DSL capability the markdown engine cannot replicate.

**Accountability:** CEO to report markdown engine ship status, contributor format breakdown, and cost-per-workflow-by-format metrics at every board meeting.

---

## 2. Community Engagement Policy

### 2.1 Founding Principle

Ash is an open-source project stewarded by a company, not a company product with an open-source license. The community's trust is the organization's primary moat. All community engagement shall be governed by the principle that contributors are owners of the project, not consumers of a product.

### 2.2 Open-Source License Commitment

Ash is and shall remain MIT-licensed, permanently and irrevocably. This commitment is codified in the repository's GOVERNANCE.md as a constitutional provision that may not be modified without a community governance process including an RFC, a public discussion period, and a contributor vote.

No license-change proposal shall be entertained by the board before Ash reaches $10M ARR and maintains at least 50 enterprise customers with annual contracts. At no revenue level may a license change proceed without the community governance process described above.

### 2.3 Community Governance Structure

The following governance infrastructure shall be published within 30 days (by 9 August 2026), simultaneously with the pricing page:

#### 2.3.1 Contributor License Agreement / Developer Certificate of Origin

A CLA or DCO policy shall be adopted to ensure clean IP provenance on all community contributions. This is a prerequisite for Series A fundraising diligence and protects the organization from IP disputes on core engine contributions.

**Accountability:** CEO to present a recommended CLA/DCO model (Apache-style CLA vs. Linux DCO) to the board by the October meeting. External open-source legal counsel shall be retained within 60 days to advise on this decision.

#### 2.3.2 Maintainer Nomination Process

A published process shall enable contributors to progress from first-time contributor to reviewer to maintainer, with defined criteria at each stage:

- **Contributor:** Has submitted at least one merged PR.
- **Reviewer:** Has demonstrated sustained contribution quality and is granted PR review privileges.
- **Maintainer:** Has earned commit access and voting rights on project decisions through sustained contribution, review activity, and community trust.

#### 2.3.3 Contributor Pathway and Onboarding

The organization shall build and publish a structured contributor pathway:

- **Good-first-issue labeling:** Every issue tagged as accessible to new contributors within 48 hours of triage.
- **Shepherd program:** Each new contributor assigned to a maintainer for code review mentorship.
- **`ash discover` contributor extension:** If a new user can find a workflow to run in 30 seconds, a new contributor shall be able to find a good-first-issue in 30 seconds. This symmetry shall be scoped and placed on the roadmap.

#### 2.3.4 Public Roadmap

A public community roadmap shall be published, scoped to Q3 and Q4 2026 deliverables actually committed — not aspirational items. Updates shall be published at the start of each quarter.

### 2.4 Contributor Compensation and Recognition

#### 2.4.1 Community Stipend Program

A contributor stipend program shall be funded at $4,000/month (reallocated from the marketing budget). This program shall compensate the top 5 contributors for their sustained community labor, including PR review, discussion responses, tutorial authorship, and community management.

#### 2.4.2 Top Contributor: @devopsjane

A paid maintainer stipend or part-time contract shall be offered to @devopsjane within two weeks of this meeting (by 23 July 2026). If she is open to a full-time role, the budgeted Developer Advocate position shall be offered to her directly. She is not free labor — she is the organization's most valuable community asset and shall be treated as such.

#### 2.4.3 Startup Program

The board directs the CEO to scope an Ash-sponsored team tier program for pre-seed and seed-stage companies, application-based and renewable annually. This program ensures that the earliest-stage organizations — the 2-3 person startups that produce the enterprise champions of tomorrow — are not priced out of Ash at the team-formation threshold.

**Accountability:** CEO to present the startup program scope at the October board meeting.

### 2.5 Community Health Metrics

The organization shall instrument and report the following community health metrics in every board pack:

| Metric | Frequency | Target |
|--------|-----------|--------|
| Contributor retention (first-to-second PR conversion) | Quarterly | >25% |
| Repeat contributor rate | Quarterly | Trend upward from 15% |
| Open PR count | Monthly | <5 |
| Average time to first response | Monthly | <24 hours |
| Top-20 contributor sentiment survey | Quarterly | Report findings |
| Contributor format breakdown (% .md vs .ash PRs) | Quarterly | Trend analysis |

### 2.6 Contributor Relationship Management

The organization shall maintain active, personal relationships with its top 20 contributors. Specific obligations:

- The CEO shall personally reach out to the top 10 contributors each quarter to understand their needs, blockers, and sentiment — reporting findings to the board.
- The organization shall immediately investigate any top-20 contributor who goes silent for more than three weeks. (@aibuilder's Q1-to-Q2 disappearance without outreach is a documented failure that shall not recur.)
- A designated team member shall own contributor relationships as a formal responsibility — initially a senior engineer, transitioning to the Developer Advocate upon hire.

### 2.7 Documentation Standards

The organization shall maintain documentation that answers the top five questions every new user asks — on the first page of the getting-started guide:

1. What agent should I use for task X? (Recommendation engine, targeting `ash discover` v2 by end of Q3.)
2. How do I run my first workflow in 5 minutes?
3. How do I integrate Ash into my CI pipeline?
4. What format should I write workflows in? (Answer: markdown. Always markdown.)
5. Where do I go for help?

**Accountability:** Getting-started guide shipping by end of July 2026. `ash discover` v2 with recommendation engine targeting end of Q3 2026. Twenty workflow templates for common use cases (CI/CD, code review, deployment, testing, onboarding) published as runnable `.md` files, funded by the content budget.

### 2.8 Communication and Transparency

#### 2.8.1 Pricing and Runway Transparency

The pricing page shall carry the organization's runway math visibly on the page — not in a separate blog post. The community shall see: "Ash burns $310,000 per month. We have 14 months of runway. The free tier is permanent and irrevocable. Here is what costs money and why."

#### 2.8.2 Founder Communications

On the day the pricing page goes live, the CEO shall publish a founder communication — in his own voice, undefended by comms review — explaining the decisions this board meeting produced: the pricing model, the licensing commitment, the markdown-first strategy, and why the organization needs revenue to survive. The tone shall be honest, transparent, and personal.

#### 2.8.3 Community Discussion

A community discussion thread shall be opened in the repository's Discussions tab on the same day pricing launches. The CEO and engineering leads shall answer questions directly and publicly. No "contact us" gate and no opaque feedback form.

#### 2.8.4 Free Tier — Irrevocable Covenant

The free tier for individual developers ("one developer, unlimited workflows, full product, forever") shall be codified in GOVERNANCE.md as irrevocable. This is a binding covenant — not a pricing-page promise subject to redesign. It may not be modified, degraded, or revoked without a community governance vote.

---

## 3. GTM Strategy Summary

### 3.1 Strategic Principle

Self-serve Ash Cloud is a GTM funnel for enterprise — not a standalone revenue story. At current usage depth and conversion rates, organic signup revenue will not close the gap against a $310,000/month burn. Enterprise contracts are the only revenue path that closes the runway math. Every GTM investment shall be evaluated against its contribution to the sequence: individual developer adoption → team adoption → enterprise contract.

### 3.2 Adoption Funnel and Required Instrumentation

The organization shall instrument and report the following funnel metrics. Every metric marked as a "data gap" must be closed by the October 2026 board meeting.

| Stage | Metric | Status |
|-------|--------|--------|
| Signup → Activation | % of signups running a workflow within 48 hours | **Data gap — instrument by October** |
| Activation → Depth | % of activated users running a second workflow within week one | **Data gap — instrument by October** |
| Individual → Team | Team-invite conversion rate; org size distribution (1, 2, 3+ users) | **Data gap — instrument by October** |
| Team → Trial | 14-day team trial conversion rate | **Must be instrumented at pricing launch** |
| Trial → Paid | Trial-to-paid conversion rate | **Must be tracked from launch** |
| Free → Enterprise | Correlation between team size and enterprise inquiry | **Data gap — instrument by October** |

### 3.3 Pricing Model

The board has adopted a per-seat pricing model as the primary monetization structure:

**Free tier:** One developer, unlimited workflow executions, full product, forever. Irrevocable. Codified in GOVERNANCE.md. This is the top of the adoption funnel and the community trust anchor.

**Team tier:** $29 per seat per month. Minimum 3 seats ($87/month). 14-day trial triggered automatically when a second developer joins an organization. Includes shared workflow dashboards, team audit trails, and CI integration.

**Enterprise tier:** Per-seat annual contracts starting at 200 seats (~$70,000/year). Volume discounts above 500 seats. Includes SSO, RBAC, VPC single-tenant deployment, EU AI Act Article 14 audit trail exports, and priority support. Published per-seat rate — a procurement analyst must be able to quote it in 60 seconds without contacting sales.

A metered self-serve Pro tier ($0.10/execution above 100 free per month) shall be maintained as an alternative for organizations that prefer usage-based billing.

The pricing page, the community governance model, and a founder communication from the CEO shall ship together within 30 days (by 9 August 2026).

**Accountability:** The 3-seat team-tier minimum shall be reviewed at the October board meeting against actual team-size distribution data. The board shall evaluate whether the minimum suppresses champion formation among 2–4 person teams. The metered enterprise committed-volume model shall be retained as a fallback enterprise SKU structure.

### 3.4 Enterprise Pipeline Management

#### 3.4.1 Champion Mapping

For every enterprise account, the organization shall maintain:
- A named internal champion, with their role and deployment status (not "evaluating" — deployed in a non-production environment).
- A named executive sponsor with budget authority.
- Current procurement gate status (security review, legal review, procurement review).
- Last contact date.

If a champion cannot be named, the account is classified as a lead, not a pipeline opportunity.

**Accountability:** CEO to deliver a champion map for all six enterprise accounts by the October board meeting. Elena Vasquez's probability-weighted ACV model ($81,500) shall be recalculated with champion-qualified accounts only.

#### 3.4.2 Pipeline Probability Thresholds

Accounts shall be classified as follows:

- **Qualified opportunity:** Named champion deployed in non-production + named budget-holder engaged.
- **Evaluation lead:** Technical team interested; no budget authority identified.
- **Lead:** Initial contact; no deployment, no champion.

Only "qualified opportunities" shall be reported as pipeline for revenue forecasting purposes.

#### 3.4.3 Sales Motion

The Account Executive hire is withdrawn. A solutions engineer shall be hired instead. This role shall:
- Run technical evaluations for enterprise accounts.
- Build champion relationships.
- Generate enablement artifacts (security architecture whitepaper, compliance overview, RFx response templates — all by end of Q3 2026).
- Monitor self-serve accounts crossing $200/month as warm enterprise leads.

The Account Executive hire shall be reconsidered only after SOC 2 Type II is in progress, a VPC deployment path exists, and the organization has 10+ qualified enterprise opportunities with named champions and budget-holder engagement.

### 3.5 Competitive Posture

The primary competitive response to TaskWeaver and any future `.md`-based competitor is product velocity — not a license change. The following form the competitive moat:

1. **Markdown engine ships by 30 September 2026** — closing the product gap TaskWeaver currently occupies as the only launched `.md`-based orchestration tool.
2. **"First workflow in 5 minutes"** as the onboarding metric. Ash shall beat TaskWeaver on time-to-value.
3. **Enterprise governance features** (SOC 2, RBAC, Article 14 audit trails, VPC deployment) — organizational assets a 3-person YC team cannot rapidly replicate.
4. **Community ownership** — contributor governance, maintainer pathways, and stipends that make Ash a project contributors feel invested in, not a product they merely use.
5. **Adoption depth** — workflows committed, CI integrated, teams trained — operational switching costs no fork can copy.

### 3.6 Marketing Budget Reallocation

The $12,000/month marketing budget is reallocated effective end of July 2026:

| Allocation | Amount | Purpose |
|------------|--------|---------|
| SOC 2 scoping fund | $4,000/month | Audit firm engagement, tooling, compliance resource |
| Developer content and templates | $4,000/month | 20 workflow templates, tutorials, community content production |
| Community stipend program | $4,000/month | Top-5 contributor stipends, including @devopsjane |

Zero dollars are allocated to paid brand awareness. Enterprise GTM content (whitepapers, security architecture documentation, compliance briefs, RFx response templates) shall be produced by the existing team as a Q3 deliverable — it does not require external spend but does require internal prioritization.

### 3.7 Developer Content Strategy

The organization shall produce and publish:

- Twenty workflow templates covering CI/CD, code review, deployment, testing, and onboarding — each published as a runnable `.md` file in a public repository.
- Tutorials: "Build a CI pipeline with Ash in 10 minutes," "Replace your Makefile with Ash," "5 agents, one workflow."
- A formal loss-review analysis of the deal lost to a bash script — understanding what was demoed, where the evaluation failed, and what must change in the sales motion to prevent recurrence. Findings to be presented to the board.

### 3.8 Churn Management

The 12% monthly churn rate shall be decomposed by driver (activation churn vs. capability churn vs. competitive churn), with an exit survey instrumented for departing users, and findings reported at the October board meeting. Churn reduction below 8% is a target for H1 2027.

---

## 4. Financial Statements

The board has reviewed the detailed financial report at `board-output/2026-07-09/finance-report.md` (dated 9 July 2026, prepared by the Office of the CFO) and formally approves it as an accurate representation of the company's financial position.

### 4.1 Key Financial Data (Incorporated by Reference)

The following data points from the approved report are incorporated into these policies:

- **Cash on hand:** $4,400,000 as of 30 June 2026.
- **Average monthly burn:** $310,000 (Q2 2026 actual).
- **Runway:** 14 months at current burn.
- **Revenue:** $400 MRR (self-serve Ash Cloud).
- **Headcount:** 18 full-time employees.
- **Enterprise pipeline:** $600,000 total estimated ACV across six accounts; probability-weighted: ~$81,500.

### 4.2 Budgetary Authorizations

The board has approved the following budgetary actions:

1. **SOC 2 Type II engagement:** $80,000–$120,000 total cost, funded from the marketing reallocation ($4,000/month) and general operating reserves. Audit firm selection and kickoff within 30 days.

2. **Marketing budget reallocation:** The existing $12,000/month marketing budget is reallocated to SOC 2 scoping ($4K), developer content and templates ($4K), and community stipends ($4K), effective end of July 2026.

3. **AE hire withdrawn; security/compliance engineer approved:** The budgeted Account Executive position ($150,000 OTE) is cancelled. The board authorizes hiring a dedicated security/compliance engineer at a comparable budget range ($120,000–$150,000 OTE) to own SOC 2 evidence collection, penetration test coordination, enterprise security questionnaires, and security architecture.

4. **Solutions engineer approved:** The board authorizes hiring a solutions engineer to run technical evaluations, build champion relationships, and generate enablement artifacts — in place of the withdrawn AE hire.

5. **Senior backend engineer:** The existing offer remains in place. Hire expected August 2026.

6. **Developer advocate:** Hiring deferred pending outreach to @devopsjane for the role. If she is not available, standard hiring process to commence.

7. **Community stipend program:** $4,000/month allocated from the reallocated marketing budget. @devopsjane's stipend or contract shall be offered within two weeks.

### 4.3 Runway Monitoring

The board directs the Investor Director (Priya Nair) to maintain a runway-risk dashboard updated for every board meeting:

1. Months of cash remaining.
2. SOC 2 certification estimated date vs. actual progress.
3. Error rate trend against the 2.0% and 0.5% targets.
4. Enterprise pipeline conversion status: accounts cleared through security review to procurement stage.
5. Markdown engine ship status against the 30 September 2026 deadline.
6. Probability of cash-out before first enterprise close, updated quarterly.

The board shall calendar a go/no-go fundraising decision for October 2026 — after at least one enterprise deal should have either closed or failed definitively.

---

## 5. Policy Governance

### 5.1 Review Cycle

These policies shall be reviewed at each quarterly board meeting. Amendments require a board resolution.

### 5.2 Accountability

The CEO is accountable for the execution of all action items and mandates specified herein. Progress shall be reported in the standardized board pack at every meeting. The board shall evaluate the CEO against the specific deliverables and timelines contained in these policies.

### 5.3 Supersession

These policies supersede any prior informal or undocumented practices inconsistent with the resolutions adopted herein. Where these policies conflict with prior board minutes, these policies govern.

---

*Approved by the Board of Directors on 9 July 2026.*

*Marcus Thorne, Executive Director & CEO*
*Priya Nair, Investor Director & Chair*
*Sarah Chen, Director, Open-Source / Community*
*Dr. Elena Vasquez, Independent Director, Enterprise Architecture*
*James Okonkwo, Independent Director, DevTools Go-To-Market*
