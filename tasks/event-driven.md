# Event Triggered Workflow Execution
Ash workflows are trigger by user's execution of command.
In typical cases, workflows are triggered by certain events.
Event triggered workflow make full automas possible, it can also help add human steps in the process.
It is a must for enterprise level orchestration

a key question here is are we adding the feature to ash application, we should create a separate application that listens to events and call ash applciatin to run workflow.

Behavior wise this application likely need to be daemon process, it could be a good candidate to be the relay layer between asyn application and telemetry data store

feature wise the application need to listen to event sources, which could be 
- folder 
- message streams (MQ , Kafka, or cloud based messaing service)
- web hook
- chat based applications


we should also consider cron like scheduled job

## configurations,
we need identify key information required for configuring event source and target workflow
it is likely this will be part of advanced user scenarios that requires centeralized store for configs

##  User Interaction
This is where manual configuration can be tidious that we might need a gui, as well as a workspace concept that have all the flows avaiable for use to select from.
the key information required can be a good guide to work out how GUI should be built



