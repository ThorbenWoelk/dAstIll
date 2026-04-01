# Tasks: BigQuery Billing Export

## Current State
Terraform now manages an optional billing export dataset, the required BigQuery APIs, and dataset access for the Google-managed billing export writers. Local validation passes; the remaining step is the one-time Cloud Billing-side export toggle after apply.

## Steps
- [x] Create the Terraform variables and local values for optional billing export prerequisites.
- [x] Add the BigQuery dataset and dataset-access resources for Cloud Billing export writers.
- [x] Extend managed project services for the BigQuery export prerequisites.
- [x] Document the final Cloud Billing-side activation step and example configuration.
- [x] Validate the Terraform configuration locally.

## Decisions Made During Implementation
- The repo will manage only supported prerequisites in Terraform.
- The Cloud Billing export toggle itself remains a documented one-time manual step.
