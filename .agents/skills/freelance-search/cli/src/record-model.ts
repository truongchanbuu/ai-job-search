import type { FreelanceOpportunityRecord } from "./models.js"

export function validateRecord(record: FreelanceOpportunityRecord): void {
  if (!record.recordId) throw new Error("Record requires recordId")
  if (!record.opportunityId) throw new Error("Record requires opportunityId")
  if (!record.status) throw new Error("Record requires status")
  if (!record.savedAt) throw new Error("Record requires savedAt")
}
