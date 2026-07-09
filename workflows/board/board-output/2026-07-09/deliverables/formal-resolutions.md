# Formal Resolutions of the Board of Directors — Ash

**Date of Meeting:** 9 July 2026
**Authority:** These resolutions constitute the legal authority for the company to execute the actions described herein, as adopted by the Board of Directors at the duly convened meeting of 9 July 2026.

---

## 1. Administrative

**1.1** RESOLVED THAT the minutes of the previous board meeting be and are hereby approved as circulated.

**1.2** RESOLVED THAT the agenda for today's meeting be and is hereby adopted.

---

## 2. Pricing and Monetization Model

**2.1** RESOLVED THAT the board adopts a per-seat pricing model as the primary monetization structure for Ash, structured as follows: (i) Free tier — one developer, unlimited workflow executions, full product, forever, irrevocable and codified in the repository's GOVERNANCE.md; (ii) Team tier — $29 per seat per month, minimum three seats ($87/month), with a 14-day trial triggered automatically when a second developer joins an organization, including shared workflow dashboards, team audit trails, and CI integration; (iii) Enterprise tier — per-seat annual contracts starting at 200 seats (~$70,000/year), volume discounts above 500 seats, including SSO, RBAC, VPC single-tenant deployment, EU AI Act Article 14 audit trail exports, and priority support, with a published per-seat rate and a self-serve pricing page that a procurement analyst can quote in 60 seconds without contacting sales. (Adopted 3–2.)

**2.2** RESOLVED THAT the pricing page, the community governance model (CLA/DCO, maintainer nomination process, contributor pathway, public roadmap), and a founder communication from the CEO explaining the runway math and the monetization and licensing decisions shall ship together within 30 days.

**2.3** RESOLVED THAT the 3-seat team-tier minimum shall be reviewed at the next board meeting against team-size distribution data, with explicit evaluation of whether the minimum suppresses champion formation among 2–4 person teams that serve as leading indicators of future enterprise conversion.

**2.4** RESOLVED THAT Elena Vasquez's startup program proposal — Ash-sponsored team tier for pre-seed and seed-stage companies, application-based, renewable annually — shall be scoped and presented at the next board meeting as a mechanism to ensure the adoption flywheel is not gated at the earliest stages of organizational formation.

**2.5** RESOLVED THAT the board retains the metered enterprise committed-volume model as a fallback enterprise SKU structure should the execution-distribution data at the next board meeting demonstrate that said structure is necessary; and that the per-seat model shall be validated or adjusted against actual usage data (execution distributions, team-size distributions, activation cohorts, churn decompositions) at the next board meeting.

**2.6** RESOLVED THAT self-serve Ash Cloud is expressly designated as a GTM funnel for enterprise — a pipeline generator and proof-of-willingness-to-pay — and is not a stand-alone revenue story at current conversion rates; and that enterprise contracts at $60,000+ ACV are the only revenue path that closes the runway math against $310,000/month burn.

---

## 3. Licensing

**3.1** RESOLVED THAT the board unanimously affirms that Ash remains MIT-licensed, permanently and irrevocably, and that this commitment shall be codified in the repository's GOVERNANCE.md as a binding project covenant — not a pricing-page promise, not a board-meeting resolution subject to quarterly review, but a constitutional provision of the project's governance model that may not be modified without a community governance process including an RFC, a public discussion period, and a contributor vote. (Adopted unanimously.)

**3.2** RESOLVED THAT no license-change proposal shall be entertained by this board before Ash reaches $10 million ARR and maintains at least 50 enterprise customers with annual contracts — the combined threshold below which the company lacks the financial resilience and customer concentration to survive the community fracture and enterprise pipeline destruction that a license change would trigger. Even above that threshold, any proposal must satisfy a community governance process including RFC, discussion period, and contributor vote.

**3.3** RESOLVED THAT the defense against competitive absorption under MIT is defined as the following five execution items, which the CEO has committed to and which the board will track at every meeting until completion: (a) the markdown engine ships as the primary product interface by end of Q3 2026; (b) SOC 2 Type II scoping begins within 30 days, audit firm selected, kickoff scheduled; (c) error rate drops below 2% by end of Q3, with a scoped path to 0.5%; (d) EU AI Act Article 14 audit trail compliance ships by January 2027; and (e) community governance model is published before or simultaneously with the pricing page within 30 days.

**3.4** RESOLVED THAT the board recognizes that the `.md` file format for expressing AI orchestration is the company's primary strategic asset, not the execution engine, and that this format is already open and MIT-licensed. The company's competitive strategy must rest on being the best implementation of the open format (velocity, reliability, enterprise governance), not on controlling access to the format through legal means.

**3.5** RESOLVED THAT the CEO shall publish, alongside the pricing page within 30 days, a founder communication that addresses the licensing question transparently — stating that MIT is permanent and irrevocable, explaining why the board considered and rejected any alternative, and making the affirmative case that Ash's moat is adoption depth, community trust, enterprise governance, and execution velocity.

**3.6** RESOLVED THAT the board notes that the per-seat monetization model and the MIT license form a single trust posture, and that the combined effect of per-seat friction at the team-formation threshold and any perceived license instability would compound to suppress the bottom-up adoption flywheel. The board's unanimous rejection of any license change mitigates the compound risk.

---

## 4. DSL Investment and Markdown Engine

**4.1** RESOLVED THAT the board adopts a markdown-first product strategy with the `.ash` DSL repositioned as an internal compile target and power-user feature, as follows: (i) The markdown engine ships as the primary product interface by September 30, 2026 — the documented default, the tutorial path, the community contribution surface, and the first experience a new user encounters. This date is a hard deadline. (ii) Engineering allocation inverts by end of July 2026: the markdown engine and context compaction receive the majority of engineering resources; the `.ash` DSL and execution engine drop to maintenance — bug fixes, critical patches, and support for existing power-user workflows, but no new features, no syntax extensions, and no investment in parser expressiveness beyond what is required to keep the compile target functioning. The DSL allocation shall not exceed 20% of engineering resources after the inversion. (iii) The DSL continues as a user-facing format for power users whose workflows require expressiveness the markdown engine cannot yet provide, documented as advanced usage, not primary onboarding. (iv) The board will evaluate sunset of the DSL as a user-facing format when: (a) the markdown engine supports expressiveness equivalent to the DSL for 95% of workflows executed on the platform; (b) the DSL's active user base drops below 5% of weekly active users; and (c) the cost per `.ash` workflow exceeds $200/month. (Adopted unanimously with qualifications.)

**4.2** RESOLVED THAT a migration tool (`ash migrate --to-markdown`) shall ship alongside the markdown engine by September 30, 2026.

**4.3** RESOLVED THAT the CEO shall personally reach out to the top 20 contributors before the markdown engine ships to communicate the format transition, explain the rationale, solicit feedback, and incorporate migration requirements.

**4.4** RESOLVED THAT the CEO shall validate, before presenting the markdown engine to any enterprise compliance team, that a non-engineer can independently read, trace, and understand the decision logic in a 20-step markdown workflow.

**4.5** RESOLVED THAT the DSL transition proceeds in two phases: Phase One (July–August 2026) — markdown engine and context compaction receive priority, DSL drops to maintenance; Phase Two (September–December 2026) — once the markdown engine ships, the engineering team scopes the expressiveness gap and builds the Q4 markdown roadmap to close it. The DSL continues at maintenance throughout.

**4.6** RESOLVED THAT the contributor format breakdown data (`.ash` vs `.md` PRs, support tickets, community templates, and format preference of top 20 contributors and new vs. repeat contributors) shall be collected and reported at every board meeting.

**4.7** RESOLVED THAT any DSL sunset decision shall be gated on enterprise contract closure — specifically, at least one enterprise customer's deployed workflows must be fully migrated to markdown — to ensure that the sunset does not disrupt revenue-generating deployments.

---

## 5. Budget and Financial

**5.1** RESOLVED THAT the $12,000/month marketing budget be reallocated effective end of month as follows: $4,000 to SOC 2 Type II scoping and audit preparation; $4,000 to developer content and workflow templates (20 templates covering CI/CD, code review, deployment, testing, and onboarding); and $4,000 to a community stipend program. Zero dollars shall be allocated to paid brand awareness until such time as the company reaches a revenue threshold that makes such spend meaningful.

**5.2** RESOLVED THAT the board approves the initiation of SOC 2 Type II scoping within 30 days, with an estimated all-in budget of $80,000–$120,000 (audit firm, tooling, and dedicated compliance resource), and authorizes the CEO to select an audit firm and schedule the kickoff within that timeframe.

**5.3** RESOLVED THAT the board approves the redirection of the Account Executive budget toward a dedicated security/compliance engineer at $120,000–$150,000 OTE, with said hire to own SOC 2 evidence collection, pentest coordination, enterprise security questionnaires, and security architecture.

**5.4** RESOLVED THAT the board approves the hiring of a solutions engineer (replacing the withdrawn AE role) at approximately $120,000–$150,000 OTE to run technical evaluations, build champion relationships, generate enablement artifacts, and monitor self-serve accounts for enterprise conversion signals.

**5.5** RESOLVED THAT the $4,000/month community stipend allocation shall include a paid maintainer stipend for @devopsjane (offer within two weeks), a broader contributor stipend program for the top 5 contributors, and tooling to instrument the contributor funnel.

**5.6** RESOLVED THAT the board recommends retaining external open-source legal counsel within 60 days with specific expertise in contributor license agreements and open-source IP provenance, to advise on the CLA/DCO decision and to prepare the company for future financing IP diligence.

---

## 6. Hiring and Personnel

**6.1** RESOLVED THAT the Account Executive job description is withdrawn, effective immediately. The AE hire shall be deferred until SOC 2 is in progress, a VPC deployment path exists, and the company has 10+ qualified enterprise opportunities with named champions and budget-holder engagement.

**6.2** RESOLVED THAT the senior backend engineer offer (outstanding, starting August) stands and shall not be rescinded.

**6.3** RESOLVED THAT two engineers on the existing team shall be designated as official PR reviewers with SLAs as part of their formal job responsibilities, effective immediately.

**6.4** RESOLVED THAT the CEO shall implement a triage rotation such that open issues receive initial classification within 48 hours.

**6.5** RESOLVED THAT the board recommends the CEO hire an operational partner — Chief of Staff or VP of Operations — within six months to build the operating rhythm, instrument metrics, manage SOC 2 program management, prepare board materials, and transition operational functions off the CEO's plate.

**6.6** RESOLVED THAT at least one other engineer must be paired on the execution engine internals, with the explicit goal that by December 31, 2026, at least one other person on the engineering team can modify the execution graph and debug a parser failure without the CEO present.

---

## 7. Product and Engineering

**7.1** RESOLVED THAT context compaction shall ship in August 2026, addressing the largest error-rate driver (truncated output at context window limits) and extending reliable workflow depth to approximately 100 steps.

**7.2** RESOLVED THAT the error rate target is sub-2% by end of Q3 2026, driven by context compaction and output schema validation, with a scoped path to 0.5% to be presented at the next board meeting.

**7.3** RESOLVED THAT EU AI Act Article 14 audit trail compliance — including confidence scoring on every agent output, decision rationale logged at every branch point, and human-override audit events with identity, timestamp, and justification — shall ship by January 2027, the enforcement date of the regulation.

**7.4** RESOLVED THAT RBAC with creator, approver, and operator role separation shall ship in Q4 2026 as a prerequisite for SOC 2 change management criteria and enterprise procurement.

**7.5** RESOLVED THAT a multi-region architecture roadmap and a data residency matrix (including EU region support) shall be presented at the next board meeting.

**7.6** RESOLVED THAT a documented disaster recovery plan with defined RTO and RPO targets shall be produced within 90 days.

**7.7** RESOLVED THAT enterprise SSO (SAML 2.0 and OpenID Connect with Azure AD and Okta) is targeted for Q4 2026, contingent on auth infrastructure stabilization.

**7.8** RESOLVED THAT the VPC single-tenant deployment SKU shall be scoped and a timeline presented at the next board meeting (estimated 4–6 months of engineering effort), prioritized as the first enterprise deployment model.

**7.9** RESOLVED THAT `ash discover` v2 with a recommendation engine — providing agent selection guidance based on community success rates — shall target end of Q3 2026.

**7.10** RESOLVED THAT a getting-started guide answering the top five questions from new users shall ship by end of month, and 20 pre-built workflow templates for the 10 most common use cases shall be published, funded by the developer content budget reallocation.

**7.11** RESOLVED THAT the 90-day reliability roadmap for Ash Cloud commits to: (i) error rate sub-2% by end of Q3; (ii) uptime target of 99.9%; and (iii) no new cloud features shipped until reliability targets are met. The early adopter experience on Ash Cloud shall be stabilized before new features are added.

**7.12** RESOLVED THAT bounded execution depth with explicit halt-on-uncertainty behavior shall be scoped into the context compaction release (August 2026): the engine must detect when quality is degrading, log the event, and halt gracefully rather than degrade silently.

---

## 8. Governance and Board Operations

**8.1** RESOLVED THAT a standardized board pack template shall be adopted within 30 days, containing at minimum: (i) Financials — cash, burn, runway, revenue (actual vs. plan), pipeline with probability-weighted ACV and executive sponsor mapping; (ii) Product metrics — activation cohorts, WAU with churn decomposition, execution volume distribution, team-size distribution, error rate trend; (iii) Community metrics — contributor retention curves, first-PR-to-second-PR conversion, open PR/issue counts with age distribution, top-20 contributor sentiment; (iv) Enterprise pipeline — named champion, deployment status, budget-holder engagement, procurement gate status for every account; (v) Engineering milestones — RAG status for markdown engine, context compaction, pricing page, SOC 2 kickoff, and community governance model.

**8.2** RESOLVED THAT Priya Nair, as investor director, shall maintain a runway-risk dashboard updated for every board meeting showing: months of cash remaining, SOC 2 certification estimated vs. actual progress, error rate trend against targets, enterprise pipeline conversion status, markdown engine ship status against the September 30 deadline, and a probability-of-cash-out-before-first-enterprise-close estimate.

**8.3** RESOLVED THAT the board shall hold quarterly meetings. The next meeting shall be scheduled for October 2026.

**8.4** RESOLVED THAT the CEO shall receive structured enterprise coaching from Dr. Elena Vasquez (procurement language and enterprise objection handling) and James Okonkwo (champion mapping and solutions-engineer-led sales motion), with the goal of independent enterprise procurement capability by Q1 2027.

**8.5** RESOLVED THAT the board does not recommend a change in CEO but recommends a transition from lead engineer to company-builder, to be evaluated at the October 2026 meeting against: inversion of engineering allocation, closure of data gaps, and demonstrated progress on SOC 2, markdown engine, pricing page, and community governance.

**8.6** RESOLVED THAT the board does not recommend changes to board composition at this time beyond the retention of external open-source legal counsel.

---

## 9. Community Governance and Contributor Programs

**9.1** RESOLVED THAT a community governance model shall be published within 30 days, including: (i) a CLA/DCO policy (CEO to review Apache-style CLA and Linux DCO model and recommend one to the board); (ii) a maintainer nomination process; (iii) a contributor pathway from first PR to maintainer; and (iv) a public community roadmap scoped to Q3 and Q4 2026 deliverables.

**9.2** RESOLVED THAT the free tier commitment — one developer, unlimited workflows, full product, forever — shall be codified in GOVERNANCE.md as irrevocable, enforceable, and not subject to modification without a community governance vote.

**9.3** RESOLVED THAT quarterly contributor health checks are established as a standing board metric, covering: contributor retention curves, sentiment surveys for top 20 contributors, first-PR-to-second-PR conversion, time-to-merge, and repeat contributor rate. These metrics shall be included in every board pack alongside financials.

**9.4** RESOLVED THAT the CEO shall personally reach out to @aibuilder by end of week to understand the reason for his departure as top Q1 contributor, and report findings to the board at the next meeting.

**9.5** RESOLVED THAT a paid maintainer stipend shall be offered to @devopsjane within two weeks, structured as either a part-time contract or a formal stipend, with a maintainer title and escalation path. If she is open to a full-time role, the developer advocate position becomes "hire @devopsjane."

**9.6** RESOLVED THAT a broader community contributor stipend program shall be established for the top 5 contributors, funded by the marketing budget reallocation.

**9.7** RESOLVED THAT the `ash discover` contributor onboarding extension shall be scoped — enabling new contributors to find a good-first-issue in 30 seconds, mirroring the 30-second workflow discovery experience for new users.

**9.8** RESOLVED THAT open PRs shall be reduced to under 5 by end of Q3 2026 and maintained at or below that level; community response time shall be maintained at or below the current 18-hour benchmark and not regress.

**9.9** RESOLVED THAT the contributor pathway — first PR, second PR, reviewer, maintainer — shall be instrumented with conversion data at each stage, and the repeat contributor rate shall be reported quarterly with a target of 25%+.

---

## 10. Enterprise Pipeline and GTM

**10.1** RESOLVED THAT a formal loss review of the bash-script deal shall be conducted, with findings presented to the board at the next meeting, including an analysis of what was demoed versus what should have been demoed.

**10.2** RESOLVED THAT a champion map for every enterprise account shall be delivered at the next board meeting, identifying: named internal champion, their role, their deployment status (personally deployed Ash in a non-production environment), and their internal influence/budget authority. Accounts for which a champion cannot be named shall be reclassified as leads, not pipeline opportunities.

**10.3** RESOLVED THAT the enterprise pipeline shall be restated at the next board meeting with executive-sponsor mapping, distinguishing between qualified opportunities (budget-holder engaged) and evaluation leads (technical team interested but no budget authority identified).

**10.4** RESOLVED THAT enterprise GTM content — a security architecture whitepaper, a compliance overview (SOC 2 roadmap, audit trail capabilities, RBAC roadmap), and an RFx response template — shall be produced by end of Q3 2026 by the existing team.

**10.5** RESOLVED THAT the enterprise champion migration assessment shall be delivered at the next board meeting: for the two enterprise accounts with named executive sponsors (FintechCo and DataCorp), identifying what workflows they run today, what format they use, and whether any depend on DSL capabilities the markdown engine does not yet support.

---

## 11. Data and Instrumentation

**11.1** RESOLVED THAT the following data gaps shall be closed by the next board meeting: (i) activation cohorts — signup-to-first-execution percentage within 48 hours, and first-to-second-execution percentage within week one; (ii) churn decomposition by driver — activation churn, capability churn, competitive churn, with an exit survey instrumented for departing users; (iii) execution volume distribution by org, with power-user cohort analysis (top decile, top 5%, top 1%); (iv) team-size distribution — number of orgs with 2+, 3+, 5+, and 10+ users, and correlation between team size and enterprise inquiry; (v) contributor retention curves by format and first-PR-to-second-PR conversion rate; (vi) team-invite conversion rate and trial-to-paid conversion rate.

**11.2** RESOLVED THAT the net WAU growth (new activated minus churned) shall be reported for the last three months, with L12M retention curves broken out by cohort.

**11.3** RESOLVED THAT a behavioral segmentation analysis of the 336 non-paying free users versus the 4 paying users shall be delivered at the next board meeting, covering: executions per month, team size, workflow complexity, agent count, and time on platform.

**11.4** RESOLVED THAT the cost-per-active-workflow-by-format metric (`.ash` vs `.md`) shall be reported at every board meeting, enabling data-driven sunset evaluation of the DSL.

**11.5** RESOLVED THAT contributor surface width — unique contributors per quarter, new contributor count, and the format of new and repeat contributors' PRs — shall be measured and reported quarterly.

---

## 12. Standing Authorities

**12.1** RESOLVED THAT the CEO is authorized to execute the budget reallocations, hiring decisions, and contractual commitments (including SOC 2 audit firm engagement and community stipend agreements) described in the foregoing resolutions, consistent with the specific dollar amounts and timeframes set forth herein.

**12.2** RESOLVED THAT the board delegates to Priya Nair (Investor Director) authority to work with the CEO to define the standardized board pack template within 30 days and to maintain the runway-risk dashboard for each board meeting.

**12.3** RESOLVED THAT the board delegates to Sarah Chen (Open-Source / Community Director) authority to collaborate with the CEO on contributor pathway design, community governance model development, and the CLA/DCO recommendation.

**12.4** RESOLVED THAT the board delegates to Dr. Elena Vasquez (Independent Director) authority to review the DR architecture document prior to broad presentation, and to provide enterprise coaching to the CEO on procurement language and enterprise objection handling.

**12.5** RESOLVED THAT the board delegates to James Okonkwo (Independent Director) authority to provide GTM coaching to the CEO on champion mapping and the solutions-engineer-led sales motion.

---

## Voting Record

| Resolution Group | Vote | For | Against | Abstain |
|---|---|---|---|---|
| Administrative (1.1–1.2) | Unanimous | 5 | 0 | 0 |
| Pricing and Monetization (2.1–2.6) | 3–2 | Priya Nair, Sarah Chen, Dr. Elena Vasquez | James Okonkwo, Marcus Thorne | 0 |
| Licensing (3.1–3.6) | Unanimous | 5 | 0 | 0 |
| DSL and Markdown Engine (4.1–4.7) | Unanimous with qualifications | 5 | 0 | 0 |
| Budget and Financial (5.1–5.6) | Unanimous | 5 | 0 | 0 |
| Hiring and Personnel (6.1–6.6) | Unanimous | 5 | 0 | 0 |
| Product and Engineering (7.1–7.12) | Unanimous | 5 | 0 | 0 |
| Governance and Board Operations (8.1–8.6) | Unanimous | 5 | 0 | 0 |
| Community Governance (9.1–9.9) | Unanimous | 5 | 0 | 0 |
| Enterprise Pipeline and GTM (10.1–10.5) | Unanimous | 5 | 0 | 0 |
| Data and Instrumentation (11.1–11.5) | Unanimous | 5 | 0 | 0 |
| Standing Authorities (12.1–12.5) | Unanimous | 5 | 0 | 0 |

**Directors in Attendance:** Marcus Thorne (Executive Director, Co-Founder & CEO), Priya Nair (Investor Director, Orchestra Ventures), Sarah Chen (Director, Open-Source / Community), James Okonkwo (Independent Director, DevTools Go-To-Market), Dr. Elena Vasquez (Independent Director, Enterprise Architecture).

**Certified by:** _________________________________ (Chair)

**Date:** _________________________________

---

*These resolutions constitute the formal, binding decisions of the Board of Directors. They provide the legal authority for the CEO and management to execute the actions described herein. No deviation from the specific terms, amounts, or timeframes set forth above is authorized without further board approval.*
