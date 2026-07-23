import type { FreelanceOpportunity, FreelanceOpportunityRecord } from "./models.js"
import { nowIso, stableId } from "./models.js"
import { defaultRecordsPath } from "./paths.js"
import { readJsonArray, upsertBy, writeJson } from "./storage.js"
import { validateRecord } from "./record-model.js"

export function loadRecords(path = defaultRecordsPath()): FreelanceOpportunityRecord[] {
  return readJsonArray<FreelanceOpportunityRecord>(path)
}

export function saveRecord(opportunity: FreelanceOpportunity, status: FreelanceOpportunityRecord["status"], materialRefs: string[] = [], notes?: string): FreelanceOpportunityRecord {
  const existing = loadRecords()
  const current = existing.find((record) => record.opportunityId === opportunity.opportunityId)
  const now = nowIso()
  const record: FreelanceOpportunityRecord = {
    recordId: current?.recordId ?? stableId("freelance-record", [opportunity.opportunityId]),
    opportunityId: opportunity.opportunityId,
    status,
    savedAt: current?.savedAt ?? now,
    updatedAt: now,
    sourceRefs: opportunity.sourceRefs,
    materialRefs: Array.from(new Set([...(current?.materialRefs ?? []), ...materialRefs])),
    platform: String(opportunity.platform),
    budget: opportunity.budget,
    notes: notes ?? current?.notes,
    outcome: current?.outcome,
  }
  validateRecord(record)
  writeJson(defaultRecordsPath(), upsertBy(existing, record, (item) => item.recordId === record.recordId))
  return record
}

export function attachExistingStatuses(opportunities: FreelanceOpportunity[]): FreelanceOpportunity[] {
  const records = loadRecords()
  return opportunities.map((opportunity) => {
    const record = records.find((item) => item.opportunityId === opportunity.opportunityId)
    return { ...opportunity, existingRecordStatus: record?.status ?? null }
  })
}
