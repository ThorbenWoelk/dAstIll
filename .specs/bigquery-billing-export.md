# BigQuery Billing Export

## Problem

Cost investigations currently depend on the Cloud Billing UI because the project has no BigQuery billing export configured. That makes it hard to answer basic questions like which SKU caused a charge on a specific date and prevents repeatable cost analysis from code or SQL.

## Goal

Provision the infrastructure prerequisites for Cloud Billing BigQuery export in Terraform so the repo owns the export dataset and access model, and document the remaining one-time Cloud Billing-side enablement step clearly.

## Requirements

- Terraform must be able to create a dedicated BigQuery dataset for billing exports when the feature is enabled.
- Terraform must manage the required BigQuery APIs for the export prerequisites.
- Terraform must grant the Google-managed billing export writers dataset access so export activation can write usage and pricing data.
- Terraform variables and examples must expose the billing export configuration without introducing secrets.
- Operations documentation must explain the final activation step and how to point the billing account at the Terraform-managed dataset.

## Non-Goals

- Replacing the Cloud Billing-side export toggle with an unsupported or undocumented API.
- Querying or transforming exported billing tables in this change.
- Applying the Terraform changes to production automatically.

## Design Considerations

- The Google provider does not expose a first-class Terraform resource for enabling Cloud Billing export itself, so the repo should own only the supported prerequisites and document the final manual step.
- The dataset should be optional and disabled by default so existing environments do not change unless explicitly opted in.
- The dataset location should default to a multi-region suitable for EU-based operations but remain configurable.

## Open Questions

- None at the moment. The only unsupported part is the Cloud Billing-side export toggle, which is intentionally left as a documented one-time step.
