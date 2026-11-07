// Optional browser E2E. Run ONLY with an isolated PostgreSQL-backed test server.
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
const base = process.env.RADAR_BROWSER_TEST_URL;
if (!base || process.env.RADAR_BROWSER_TEST_ALLOW_WRITES !== "yes-isolated")
  throw new Error(
    "Set RADAR_BROWSER_TEST_URL and RADAR_BROWSER_TEST_ALLOW_WRITES=yes-isolated for a disposable local server.",
  );
const url = new URL(base);
assert(
  ["127.0.0.1", "localhost"].includes(url.hostname) &&
    url.port &&
    url.port !== "8080",
  "Use an isolated loopback test port, never the normal application.",
);
const owner = "synthetic-owner-token-for-web-tests-only";
const reader = "synthetic-reader-token-for-web-tests-only";
const { chromium } = await import(
  process.env.RADAR_PLAYWRIGHT_MODULE || "playwright"
);
async function api(path, body, token = owner, method) {
  const response = await fetch(base + path, {
    method: method || (body === undefined ? "GET" : "POST"),
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const data = await response.json();
  assert(
    response.ok,
    `${path}: HTTP ${response.status} ${JSON.stringify(data)}`,
  );
  return data;
}
async function login(page, token = owner) {
  const link = await api("/v1/web-access", undefined, token);
  await page.goto(`${base}/#connect=${link.code}`);
  await page
    .getByRole("heading", { name: "Tập trung vào cơ hội phù hợp" })
    .waitFor();
}
async function complete(jobId, kind) {
  for (let i = 0; i < 40; i++) {
    const h = await api(`/v1/jobs/${jobId}/ai`);
    const op = h.operations.find((o) => o.kind === kind);
    if (op?.status === "failed")
      throw new Error(`Mock ${kind} failed: ${op.error}`);
    if (op?.status === "completed") return op;
    await new Promise((r) => setTimeout(r, 150));
  }
  throw new Error(`Mock ${kind} timed out`);
}
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  viewport: { width: 1440, height: 1000 },
  permissions: ["clipboard-read", "clipboard-write"],
});
const page = await context.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
page.on("dialog", (d) => d.accept());
const artifacts = "/tmp/radar-web-evidence";
await mkdir(artifacts, { recursive: true, mode: 0o700 });
try {
  await api("/v1/imports", {
    idempotency_key: crypto.randomUUID(),
    source: "mock",
    jobs: null,
  });
  const originals = await api("/v1/jobs?page_size=20");
  const fixture = structuredClone(
    originals.items.find((j) => j.evaluation.verdict === "apply_candidate")
      ?.job || originals.items[0].job,
  );
  fixture.provider = "manual";
  fixture.source_id = "synthetic-browser-workflow";
  fixture.url = "https://example.test/synthetic-browser-workflow";
  fixture.title = "Synthetic browser workflow <tag>";
  fixture.description =
    (fixture.description || "Build a small backend API.") +
    '\n<img src=x onerror="window.radarInjection=true">';
  await api("/v1/imports", {
    idempotency_key: crypto.randomUUID(),
    source: "manual",
    jobs: [fixture],
  });
  const job = (await api("/v1/jobs?search=Synthetic%20browser%20workflow"))
    .items[0];
  assert(job);
  await login(page);
  await page.screenshot({ path: `${artifacts}/dashboard.png`, fullPage: true });
  assert(
    !(await page.evaluate(() => location.hash)).includes("connect="),
    "One-time code removed from address bar",
  );
  assert.equal(
    await page.evaluate(() => sessionStorage.getItem("radar.web.token")),
    null,
  );
  assert(
    !(await page.evaluate(() => document.cookie)).includes("radar_web_session"),
  );
  const authCookie = (await context.cookies()).find(
    (c) => c.name === "radar_web_session",
  );
  assert(authCookie?.httpOnly && authCookie.sameSite === "Strict");
  await page.reload();
  await page
    .getByRole("heading", { name: "Tập trung vào cơ hội phù hợp" })
    .waitFor();
  await page.getByRole("link", { name: "Jobs", exact: true }).click();
  await page.getByLabel("Tìm theo từ khóa").fill("Synthetic browser workflow");
  await page.getByRole("button", { name: "Tìm jobs", exact: true }).click();
  await page.getByRole("link", { name: fixture.title, exact: true }).click();
  await page.getByRole("heading", { name: "Mô tả công việc" }).waitFor();
  assert.equal(await page.evaluate(() => window.radarInjection), undefined);
  assert.equal(await page.locator(".description img").count(), 0);
  await page.getByLabel("Ghi chú riêng").fill("Synthetic explicit preference");
  await page.getByRole("button", { name: "Interested", exact: true }).click();
  await page
    .getByRole("status")
    .filter({ hasText: "Đã lưu feedback" })
    .waitFor();
  assert.equal((await api(`/v1/jobs/${job.id}`)).decision, "interested");
  await page.getByRole("button", { name: "Đánh giá lại", exact: true }).click();
  await complete(job.id, "evaluate");
  await page.getByRole("button", { name: "Làm mới", exact: true }).click();
  await page
    .getByRole("button", { name: "Tôi đã xem · Duyệt đánh giá", exact: true })
    .click();
  await page.getByRole("button", { name: "Tạo draft", exact: true }).waitFor();
  await page.getByRole("button", { name: "Tạo draft", exact: true }).click();
  await complete(job.id, "proposal");
  await page.getByRole("button", { name: "Làm mới", exact: true }).click();
  await page.locator("#draft-text").waitFor();
  const edited =
    (await page.locator("#draft-text").inputValue()) +
    "\n\nSynthetic manual edit.";
  await page.locator("#draft-text").fill(edited);
  await page
    .getByRole("button", { name: "Lưu chỉnh sửa", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Đã lưu chỉnh sửa draft" })
    .waitFor();
  assert.equal(
    (await api(`/v1/jobs/${job.id}/ai`)).drafts[0].user_text,
    edited,
  );
  assert.notEqual(
    (await api(`/v1/jobs/${job.id}`)).application_status,
    "submitted",
  );
  await page
    .getByRole("button", { name: "Copy nội dung", exact: true })
    .click();
  assert.equal(
    await page.evaluate(() => navigator.clipboard.readText()),
    edited,
  );
  await page.screenshot({
    path: `${artifacts}/job-detail.png`,
    fullPage: true,
  });
  await page.getByRole("link", { name: "Hồ sơ", exact: true }).click();
  await page.locator("#profile-document").waitFor();
  await page
    .locator("#profile-document")
    .fill(
      (await page.locator("#profile-document").inputValue()) +
        "\nSynthetic browser profile note.\n",
    );
  await page
    .getByLabel(
      "Tôi đã kiểm tra tính chính xác và quyền chia sẻ các thông tin này.",
    )
    .check();
  await page
    .getByRole("button", { name: "Lưu phiên bản hồ sơ", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Đã lưu phiên bản hồ sơ mới" })
    .waitFor();
  assert(
    (await api("/v1/profile")).profile.documents.identity.includes(
      "Synthetic browser profile note.",
    ),
  );
  await page.getByRole("link", { name: "Mẫu proposal", exact: true }).click();
  await page.locator("#tpl-name").waitFor();
  await page.locator("#tpl-name").fill("Synthetic concise template");
  await page
    .getByRole("button", { name: "Lưu mẫu vào cấu hình", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Đã lưu thư viện mẫu" })
    .waitFor();
  assert.equal(
    (await api("/v1/config")).effective.radar.proposals.templates.items[0].name,
    "Synthetic concise template",
  );
  await page.getByRole("link", { name: "Lịch sử", exact: true }).click();
  await page.getByRole("heading", { name: "Feedback gần đây" }).waitFor();
  await page.getByRole("link", { name: "Cài đặt", exact: true }).click();
  await page.locator("#config-json").waitFor();
  const revision = (await api("/v1/config")).revision;
  await page.locator("#config-json").fill("{broken");
  await page.locator("#confirm-config").check();
  await page
    .getByRole("button", { name: "Kiểm tra & lưu cấu hình", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "JSON chưa hợp lệ" })
    .waitFor();
  assert.equal((await api("/v1/config")).revision, revision);
  assert.equal(await page.locator("#config-json").inputValue(), "{broken");
  // Real 409: another owner changes the config after this browser loaded it.
  await page
    .locator("#config-json")
    .fill(JSON.stringify((await api("/v1/config")).effective.radar));
  const config = await api("/v1/config");
  await api(
    "/v1/config",
    {
      expected_revision: config.revision,
      radar: config.effective.radar,
      scoring: null,
      notifications: null,
    },
    owner,
    "PUT",
  );
  await page
    .getByRole("button", { name: "Kiểm tra & lưu cấu hình", exact: true })
    .click();
  await page
    .getByRole("status")
    .filter({ hasText: "Dữ liệu đã thay đổi" })
    .waitFor();
  assert((await page.locator("#config-json").inputValue()).length > 10);
  // Cancelling navigation keeps the editor, route and heading consistent.
  page.removeAllListeners("dialog");
  page.once("dialog", (dialog) => dialog.dismiss());
  const retained = await page.locator("#config-json").inputValue();
  await page.getByRole("link", { name: "Jobs", exact: true }).click();
  assert.equal(await page.locator("#page-title").textContent(), "Cài đặt");
  assert.equal(await page.locator("#config-json").inputValue(), retained);
  page.on("dialog", (dialog) => dialog.accept());
  // Connection failure must preserve edits and never display success.
  await page.route("**/v1/config", (route) =>
    route.request().method() === "PUT"
      ? route.abort("failed")
      : route.continue(),
  );
  await page
    .getByRole("button", { name: "Kiểm tra & lưu cấu hình", exact: true })
    .click();
  await page.getByRole("status").filter({ hasText: "Mất kết nối" }).waitFor();
  await page.unroute("**/v1/config");
  await page.getByRole("button", { name: "Làm mới", exact: true }).click();
  await page.locator("#config-json").waitFor();
  for (const view of [
    "Tổng quan",
    "Jobs",
    "Hồ sơ",
    "Mẫu proposal",
    "Lịch sử",
    "Cài đặt",
  ]) {
    await page.getByRole("link", { name: view, exact: true }).click();
    await page.locator(".loading").waitFor({ state: "hidden" });
    assert.equal(
      await page
        .getByRole("heading", { name: "Chưa tải được dữ liệu", exact: true })
        .count(),
      0,
      view,
    );
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("link", { name: "Tổng quan", exact: true }).click();
  await page
    .getByRole("heading", { name: "Tập trung vào cơ hội phù hợp" })
    .waitFor();
  assert(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth + 1,
    ),
    "No horizontal page overflow on mobile",
  );
  await page.screenshot({ path: `${artifacts}/mobile.png`, fullPage: true });
  const readContext = await browser.newContext();
  const readPage = await readContext.newPage();
  await login(readPage, reader);
  await readPage.goto(`${base}/#job/${job.id}`);
  await readPage.getByRole("heading", { name: "Mô tả công việc" }).waitFor();
  assert(
    await readPage
      .getByRole("button", { name: "Interested", exact: true })
      .isDisabled(),
  );
  const blocked = await readContext.request.post(`${base}/v1/scheduler/scan`, {
    headers: { "X-Radar-Web": "1" },
    data: { idempotency_key: crypto.randomUUID() },
  });
  assert.equal(blocked.status(), 403);
  const csrf = await context.request.post(`${base}/v1/scheduler/scan`, {
    data: { idempotency_key: crypto.randomUUID() },
  });
  assert.equal(csrf.status(), 403);
  await readPage.route("**/web/logout", (route) => route.abort("failed"));
  await readPage
    .getByRole("button", { name: "Đăng xuất", exact: true })
    .click();
  await readPage
    .getByRole("status")
    .filter({ hasText: "Chưa thể đăng xuất" })
    .waitFor();
  await readPage.unroute("**/web/logout");
  await readPage
    .getByRole("button", { name: "Đăng xuất", exact: true })
    .click();
  await readPage
    .getByRole("heading", { name: "Mở Radar từ desktop" })
    .waitFor();
  assert.equal(await readPage.locator('input[type="password"]').count(), 0);
  const revoked = await readContext.request.get(`${base}/v1/session`);
  assert.equal(revoked.status(), 401);
  await readContext.close();
  assert.deepEqual(errors, []);
  console.log(
    "PASS: real browser + isolated PostgreSQL: navigation, XSS escaping, feedback, mock AI approval/draft/edit/copy, profile, templates, malformed config, stale update, disconnect, read-only authorization, HttpOnly cookies, reload persistence, CSRF rejection, logout revocation and mobile layout.",
  );
  console.log(`Synthetic screenshots: ${artifacts}`);
} catch (error) {
  await page
    .screenshot({ path: `${artifacts}/failure.png`, fullPage: true })
    .catch(() => {});
  throw error;
} finally {
  await browser.close();
}
