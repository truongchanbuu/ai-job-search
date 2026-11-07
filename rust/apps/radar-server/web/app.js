// Thin same-origin client. Eligibility, evidence validation and persistence belong to Rust.
const $ = (selector) => document.querySelector(selector);
const main = $("#main");
const titles = {
  dashboard: "Tổng quan",
  jobs: "Jobs",
  profile: "Hồ sơ",
  templates: "Mẫu proposal",
  activity: "Lịch sử",
  settings: "Cài đặt",
};
// Remove credentials saved by the early browser prototype. Sessions now use HttpOnly cookies.
sessionStorage.removeItem("radar.web.token");
sessionStorage.removeItem("radar.web.write");
const state = {
  connected: false,
  write: false,
  config: null,
  job: null,
  ai: null,
  profile: null,
  module: "identity",
  dirty: false,
  busy: false,
  page: 1,
  search: "",
  verdict: "",
  template: 0,
  route: 0,
};
const esc = (value) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ],
  );
const labels = {
  employed: "Đang có việc chính",
  available: "Có thể nhận việc",
  busy: "Đang bận",
  paused: "Tạm dừng",
  apply_candidate: "Phù hợp",
  review: "Cần xem thêm",
  skip: "Bỏ qua",
  save: "Đã lưu",
  interested: "Quan tâm",
  completed: "Hoàn tất",
  running: "Đang chạy",
  pending: "Đang chờ",
  failed: "Thất bại",
  strong_candidate: "Ứng viên phù hợp",
  worth_investigating: "Đáng tìm hiểu",
  low_priority: "Ưu tiên thấp",
  requires_clarification: "Cần làm rõ",
  high: "Cao",
  medium: "Trung bình",
  low: "Thấp",
  fixed: "Trọn gói",
  hourly: "Theo giờ",
  unknown: "Chưa rõ",
  none: "Chưa ứng tuyển",
  drafted: "Đã có draft",
  submitted: "Đã tự gửi",
  interviewing: "Phỏng vấn",
  hired: "Được thuê",
  rejected: "Không được chọn",
  withdrawn: "Đã rút",
  reauthorization_required: "Cần đăng nhập lại",
  manual_only: "Chỉ chạy thủ công",
};
const label = (value) =>
  labels[value] || String(value ?? "Chưa rõ").replaceAll("_", " ");
const date = (value) =>
  value ? new Date(value).toLocaleString("vi-VN") : "Chưa có";
const score = (value) => (value == null ? "—" : Number(value).toFixed(1));
const badge = (value, kind = "") =>
  `<span class="badge ${esc(kind)}">${esc(label(value))}</span>`;
const list = (values) =>
  values?.length
    ? `<ul class="list">${values.map((v) => `<li>${esc(v)}</li>`).join("")}</ul>`
    : '<p class="muted">Chưa có thông tin.</p>';
const line = (name, value) =>
  `<div class="status-line"><span class="muted">${esc(name)}</span><span>${esc(value)}</span></div>`;
const disabled = () => (state.write ? "" : " disabled");
function message(text, error = false) {
  const el = $("#message");
  el.textContent = text;
  el.className = error ? "error" : "";
  el.hidden = !text;
}
function show(html) {
  main.innerHTML = html;
  main
    .querySelectorAll("textarea,input:not([type=search]),select")
    .forEach((el) =>
      el.addEventListener("input", () => {
        state.dirty = true;
      }),
    );
}
async function api(path, body, method) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 45000);
  try {
    const response = await fetch(path, {
      method: method || (body === undefined ? "GET" : "POST"),
      headers: {
        "Content-Type": "application/json",
        "X-Radar-Web": "1",
      },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: controller.signal,
      cache: "no-store",
      credentials: "same-origin",
      redirect: "error",
    });
    const data = await response.json().catch(() => null);
    if (!response.ok)
      throw new Error(
        response.status === 409
          ? "Dữ liệu đã thay đổi trên server. Nội dung đang sửa được giữ lại; hãy sao chép trước khi làm mới."
          : response.status === 401
            ? "Phiên kết nối đã hết hạn. Sao chép phần đang sửa, rồi mở lại Radar hoặc đăng nhập lại."
            : response.status === 403
              ? "Tài khoản chỉ được xem; không có quyền thay đổi dữ liệu."
              : data?.error?.message || `HTTP ${response.status}`,
      );
    return data;
  } catch (error) {
    if (error.name === "AbortError" || error instanceof TypeError)
      throw new Error(
        "Mất kết nối hoặc quá thời gian chờ. Kết quả thao tác có thể chưa rõ; hãy làm mới để kiểm tra trước khi gửi lại.",
      );
    throw error;
  } finally {
    clearTimeout(timer);
  }
}
async function action(work) {
  if (state.busy) return;
  state.busy = true;
  document.body.setAttribute("aria-busy", "true");
  const buttons = [...main.querySelectorAll("button")].filter(
    (b) => !b.disabled,
  );
  buttons.forEach((b) => (b.disabled = true));
  try {
    await work();
  } catch (error) {
    message(error.message, true);
  } finally {
    state.busy = false;
    document.body.removeAttribute("aria-busy");
    buttons.forEach((b) => {
      if (b.isConnected) b.disabled = false;
    });
  }
}
function setSession(value) {
  state.connected = true;
  state.write = value.can_write;
  $("#connection").textContent = value.can_write
    ? "Đã kết nối · Chủ sở hữu"
    : "Đã kết nối · Chỉ xem";
}
async function connect(code) {
  setSession(await api("/web/connect", { code }));
}
function login() {
  $("#page-title").textContent = "Kết nối Radar";
  document
    .querySelectorAll("nav a")
    .forEach((a) => a.removeAttribute("aria-current"));
  show(
    `<section class="card login"><span class="eyebrow">KHÔNG GIAN CÁ NHÂN</span><h2>Mở Radar từ desktop</h2><p>Mở icon <strong>Freelance Radar</strong> trên desktop để đăng nhập tự động. Ứng dụng dùng cấu hình riêng trên máy để đăng nhập tự động.</p><div class="notice">Nếu vừa khởi động backend, hãy đợi cửa sổ trình duyệt được mở tự động. Khi phiên hết hạn, mở lại icon để kết nối lại.</div><button id="resume-session">Kiểm tra lại phiên</button></section>`,
  );
  $("#resume-session").onclick = () =>
    action(async () => {
      setSession(await api("/v1/session"));
      state.dirty = false;
      message("");
      await route();
    });
}
function safeLink(value) {
  try {
    const u = new URL(value);
    return ["http:", "https:"].includes(u.protocol) &&
      !u.username &&
      !u.password
      ? u.href
      : null;
  } catch {
    return null;
  }
}
async function dashboard() {
  const [d, s, u] = await Promise.all([
    api("/v1/dashboard"),
    api("/v1/scheduler"),
    api("/v1/upwork"),
  ]);
  return `<div class="stats">${[
    ["Jobs đã lưu", d.jobs],
    ["Ứng viên phù hợp", d.candidates],
    ["Cần xem thêm", d.reviews],
    ["Feedback", d.feedback],
  ]
    .map(
      ([n, v]) =>
        `<div class="stat"><span>${esc(n)}</span><strong>${esc(v)}</strong></div>`,
    )
    .join(
      "",
    )}</div><div class="grid"><section class="card"><h2>Tập trung vào cơ hội phù hợp</h2><p class="muted">Bắt đầu từ jobs đã xếp hạng, kiểm tra khả năng thực hiện rồi mới chuẩn bị proposal.</p><div class="actions"><a href="#jobs">Xem danh sách jobs →</a></div>${line("Chế độ", label(d.mode))}${line("Lần nhập gần nhất", date(d.last_import))}${line("Lịch mock tiếp theo", date(s.next_scan))}${line("Workers đang hoạt động", s.active_workers)}<p class="muted"><small>Không có worker thì yêu cầu scan đang chờ sẽ chưa chạy.</small></p></section><section class="card"><h2>Nguồn job</h2>${d.sources.map((v) => `<div class="status-line"><span>${esc(v.name)}</span>${badge(v.enabled ? "Đã bật" : "Đã tắt", v.enabled ? "good" : "")}</div><small>${esc(v.diagnostic)}</small>`).join("")}${line("Upwork", label(u.authentication))}<div class="notice">Upwork chỉ tìm khi bạn yêu cầu. AI và gửi proposal không tự chạy.</div></section></div><section class="card"><h2>Luồng làm việc</h2><p>Chọn job → kiểm tra điểm và rủi ro → đánh giá AI nếu cần → duyệt đánh giá → tạo draft → tự chỉnh và gửi.</p><div class="actions"><a href="#profile">Kiểm tra hồ sơ</a><span class="muted">·</span><a href="#templates">Chọn cách viết proposal</a></div></section>`;
}
async function jobs() {
  const params = new URLSearchParams({
    page: state.page,
    page_size: 20,
    sort: "score",
  });
  if (state.search) params.set("search", state.search);
  if (state.verdict) params.set("verdict", state.verdict);
  const data = await api(`/v1/jobs?${params}`);
  return `<form id="job-search" class="toolbar"><div class="search"><label for="search">Tìm theo từ khóa</label><input id="search" type="search" value="${esc(state.search)}" placeholder="Ví dụ: Rust, backend, API…"></div><div><label for="verdict">Đánh giá cơ bản</label><select id="verdict"><option value="">Tất cả</option>${[
    ["apply_candidate", "Phù hợp"],
    ["review", "Cần xem thêm"],
    ["skip", "Bỏ qua"],
  ]
    .map(
      ([v, n]) =>
        `<option value="${v}" ${state.verdict === v ? "selected" : ""}>${n}</option>`,
    )
    .join(
      "",
    )}</select></div><button>Tìm jobs</button></form><section class="card"><div class="row"><h2>${esc(data.total)} jobs</h2><span class="muted">Xếp theo ranking hiện tại</span></div>${data.items.length ? `<div class="table-wrap"><table><thead><tr><th>Job</th><th>Điểm gốc / ranking</th><th>Đánh giá</th><th>Quyết định</th></tr></thead><tbody>${data.items.map((v) => `<tr><td><a href="#job/${esc(v.id)}">${esc(v.job.title)}</a><small>${esc(v.job.provider)} · ${esc(label(v.job.job_type))} · ${v.job.budget ? esc(`${v.job.budget.amount} ${v.job.budget.currency}`) : "Ngân sách chưa rõ"}</small></td><td><strong>${score(v.evaluation.score)}</strong><small>Ranking ${score(v.personalization?.final_score ?? v.evaluation.score)}</small></td><td>${badge(v.evaluation.verdict, v.evaluation.verdict === "apply_candidate" ? "good" : v.evaluation.verdict === "skip" ? "bad" : "warn")}</td><td>${badge(v.decision || "Chưa chọn")}<small>${esc(label(v.application_status))}</small></td></tr>`).join("")}</tbody></table></div>` : `<div class="empty"><h2>Chưa có job phù hợp bộ lọc</h2><p>Xóa bộ lọc hoặc chạy một lần tìm thủ công trong Cài đặt.</p></div>`}<div class="actions pagination"><button id="prev" class="secondary" ${state.page <= 1 ? "disabled" : ""}>← Trước</button><span>Trang ${esc(state.page)}</span><button id="next" class="secondary" ${state.page * 20 >= data.total ? "disabled" : ""}>Sau →</button></div></section>`;
}
const reasons = [
  ["technology_mismatch", "Công nghệ không khớp"],
  ["too_difficult", "Quá khó"],
  ["not_interesting", "Không hứng thú"],
  ["budget_too_low", "Ngân sách thấp"],
  ["deadline_unrealistic", "Deadline thiếu thực tế"],
  ["schedule_conflict", "Không phù hợp lịch"],
  ["scope_unclear", "Scope chưa rõ"],
  ["client_risk", "Rủi ro khách hàng"],
  ["too_competitive", "Cạnh tranh cao"],
  ["better_opportunity_available", "Có cơ hội tốt hơn"],
  ["other", "Khác"],
];
function analysisCard(operation) {
  if (!operation)
    return '<p class="muted">Chưa đánh giá AI. Điểm cơ bản vẫn có hiệu lực.</p>';
  const a = operation.output;
  return `<p>${badge(operation.status, operation.status === "completed" ? "good" : "warn")} ${badge(operation.stale ? "Đã cũ" : operation.approved ? "Đã duyệt" : "Chưa duyệt")} <small>${esc(operation.provider)} / ${esc(operation.model)}</small></p>${operation.error ? `<p class="inline-error">${esc(operation.error)}</p>` : ""}${a ? `<h3>${esc(label(a.recommendation))}</h3><p>${esc(a.technical_fit)}</p>${line("Độ tin cậy", label(a.confidence))}${line("Ước lượng giờ (không cam kết)", `${a.estimated_effort?.best_case_hours ?? "?"} – ${a.estimated_effort?.conservative_hours ?? "?"}`)}<h3>Điểm mạnh có bằng chứng</h3>${list(a.strengths)}<h3>Thiếu sót & chưa rõ</h3>${list([...(a.gaps || []), ...(a.unknowns || [])])}<h3>Rủi ro</h3>${list(a.risks)}<details><summary>Giả định, chi phí và bằng chứng</summary>${list(a.assumptions)}${list(a.pricing_considerations)}${list(a.evidence_references)}</details><p><strong>Bước tiếp:</strong> ${esc(a.suggested_next_action)}</p>` : ""}${operation.status === "completed" && !operation.stale && !operation.approved ? `<button id="approve-ai"${disabled()}>Tôi đã xem · Duyệt đánh giá</button>` : ""}${operation.status === "running" ? `<button id="cancel-ai" class="secondary"${disabled()}>Hủy đánh giá</button><p class="muted">Đang xử lý. Nhấn Làm mới để xem kết quả.</p>` : ""}`;
}
async function jobDetails(id) {
  const generation = state.route;
  const [job, ai, config] = await Promise.all([
    api(`/v1/jobs/${encodeURIComponent(id)}`),
    api(`/v1/jobs/${encodeURIComponent(id)}/ai`),
    api("/v1/config"),
  ]);
  if (generation !== state.route) throw new Error("Navigation changed");
  state.job = job;
  state.ai = ai;
  state.config = config;
  const j = job.job,
    e = job.evaluation,
    p = job.personalization;
  const evaluation = ai.operations.find((o) => o.kind === "evaluate");
  const draft = ai.drafts[0];
  const link = safeLink(j.url);
  const templates = config.effective.radar.proposals.templates?.items || [];
  return `<a class="back" href="#jobs">← Danh sách jobs</a><section class="card"><div class="row"><div><h2>${esc(j.title)}</h2><span class="muted">${esc(j.provider)} · ${esc(j.category)} · ${esc(label(j.job_type))}</span></div>${badge(e.verdict, e.verdict === "skip" ? "bad" : "good")}</div><div class="tags">${j.skills.map((v) => badge(v)).join("")}</div>${link ? `<a href="${esc(link)}" target="_blank" rel="noopener noreferrer">Mở bài đăng gốc ↗</a>` : ""}</section><div class="detail-grid"><div><section class="card"><h2>Mô tả công việc</h2><div class="description">${esc(j.description || "Nguồn chưa trả mô tả.")}</div></section><section class="card"><div class="row"><h2>Đánh giá AI</h2><button id="evaluate"${!config.effective.radar.ai.runtime?.enabled ? " disabled" : disabled()}>Đánh giá lại</button></div><p class="muted"><small>Gọi provider đã cấu hình, có thể phát sinh phí. Không thay đổi điều kiện bắt buộc.</small></p>${analysisCard(evaluation)}</section><section class="card"><h2>Proposal draft</h2><p class="muted">Cần một đánh giá hiện tại đã được bạn duyệt. Draft không đồng nghĩa đã ứng tuyển.</p><label for="proposal-template">Mẫu viết</label><select id="proposal-template"><option value="">Mặc định theo cấu hình</option>${templates.map((t) => `<option value="${esc(t.id)}">${esc(t.name)}</option>`).join("")}</select><div class="actions"><button id="generate" ${!evaluation?.approved || evaluation?.stale ? "disabled" : disabled()}>${draft ? "Tạo draft mới" : "Tạo draft"}</button></div>${draft ? `<label for="draft-text">Nội dung · revision ${esc(draft.revision)}</label><textarea id="draft-text" class="draft"${disabled()}>${esc(draft.user_text ?? draft.ai_text)}</textarea><p><small>${draft.approved_text ? "Đã có bản được duyệt; sửa mới cần duyệt lại." : "Chưa duyệt bản cuối."} Bản AI và lịch sử chỉnh sửa vẫn được giữ.</small></p><div class="actions"><button id="save-draft"${disabled()}>Lưu chỉnh sửa</button><button id="approve-draft" class="secondary"${disabled()}>Duyệt bản cuối</button><button id="copy-draft" class="secondary">Copy nội dung</button></div><details><summary>Lịch sử draft (${ai.drafts.length})</summary>${ai.drafts.map((v) => `<h3>Revision ${esc(v.revision)}</h3><pre>${esc(v.user_text ?? v.ai_text)}</pre>`).join("")}</details>` : '<p class="muted">Chưa có draft. Không có thao tác gửi lên marketplace trong ứng dụng.</p>'}${ai.operations
    .filter((o) => o.kind === "proposal")
    .slice(0, 3)
    .map(
      (o) =>
        `<p>${badge(o.status)} ${esc(o.error || "")} ${o.status === "running" ? "· Nhấn Làm mới để xem kết quả." : ""}</p>`,
    )
    .join(
      "",
    )}</section></div><div><section class="card"><h2>Điểm & khả năng thực hiện</h2><p class="score">${score(e.score)} <small>điểm gốc</small></p>${line("Điều chỉnh cá nhân", score(p?.adjustment ?? 0))}${line("Ranking hiển thị", score(p?.final_score ?? e.score))}${line("Ngân sách", j.budget ? `${j.budget.amount} ${j.budget.currency}` : "Chưa rõ")}${line("Khối lượng / tuần", j.weekly_hours == null ? "Chưa rõ" : `${j.weekly_hours} giờ`)}${line("Ứng tuyển", label(job.application_status))}${e.vetoes.length ? `<h3>Điều kiện không đạt</h3>${list(e.vetoes)}` : ""}<details><summary>Chi tiết điểm cơ bản</summary>${Object.entries(
    e.breakdown,
  )
    .map(([k, v]) => line(label(k), score(v)))
    .join(
      "",
    )}${list(e.strengths)}${list(e.gaps)}${list(e.unknowns)}</details>${p ? `<details><summary>Vì sao ranking thay đổi?</summary>${line("Số bằng chứng", p.evidence_count)}${line("Độ tin cậy", score(p.confidence))}<pre>${esc(JSON.stringify(p.contributions, null, 2))}</pre><p>${esc(p.diagnostic)}</p></details>` : ""}</section><section class="card"><h2>Quyết định của bạn</h2><p>${badge(job.decision || "Chưa chọn")}</p><label for="reason-code">Lý do (tùy chọn)</label><select id="reason-code"><option value="">Không chọn lý do</option>${reasons.map(([v, n]) => `<option value="${v}">${n}</option>`).join("")}</select><label for="reason">Ghi chú riêng</label><textarea id="reason" rows="3" maxlength="4000" placeholder="Điều gì làm job phù hợp hoặc chưa phù hợp?"></textarea><div class="actions"><button data-feedback="interested"${disabled()}>Interested</button><button data-feedback="save" class="secondary"${disabled()}>Save</button><button data-feedback="skip" class="secondary"${disabled()}>Skip</button></div><details><summary>Cập nhật kết quả ứng tuyển thủ công</summary><label for="outcome">Kết quả</label><select id="outcome">${["applied", "interview", "hired", "rejected", "completed", "withdrawn"].map((v) => `<option>${v}</option>`).join("")}</select><label for="submitted-at">Thời điểm đã gửi (chỉ khi Applied)</label><input id="submitted-at" type="datetime-local"><label class="check"><input id="confirm-applied" type="checkbox">Tôi xác nhận đã tự gửi proposal trên Upwork.</label><button id="save-outcome" class="secondary"${disabled()}>Ghi nhận kết quả</button></details></section></div></div>`;
}
const modules = {
  identity: [
    "Thông tin & kỹ năng",
    "Kỹ năng, loại kinh nghiệm và bằng chứng xác minh.",
  ],
  mindset: ["Định hướng", "Cách làm việc và mục tiêu freelance."],
  availability: ["Thời gian", "Khả năng nhận việc và lịch làm việc."],
  "job-preferences": [
    "Job mong muốn",
    "Loại công việc, ngân sách và các giới hạn.",
  ],
  portfolio: [
    "Portfolio & bằng chứng",
    "Giữ rõ kinh nghiệm chuyên môn, dự án cá nhân và học thuật.",
  ],
  "proposal-style": [
    "Phong cách proposal",
    "Ngắn gọn, giải quyết đúng vấn đề, không phóng đại.",
  ],
  "lessons-learned": ["Bài học", "Ghi chú riêng từ các lần làm việc trước."],
};
function profileEditor() {
  const docs = state.profile.profile.documents;
  return `<section class="card"><div class="row"><div><h2>Hồ sơ cá nhân</h2><p class="muted">Nguồn thông tin thật cho scoring và proposal · revision ${esc(state.profile.revision)}</p></div>${badge(state.profile.source)}</div><div class="tabs">${Object.entries(
    modules,
  )
    .map(
      ([k, [n]]) =>
        `<button data-module="${k}" aria-pressed="${state.module === k}">${n}</button>`,
    )
    .join(
      "",
    )}</div><h3>${modules[state.module][0]}</h3><p class="muted">${modules[state.module][1]} Chỉnh Markdown; giữ cấu trúc YAML ở đầu tài liệu nếu có.</p><label for="profile-document" class="visually-hidden">Nội dung hồ sơ</label><textarea id="profile-document" class="editor"${disabled()}>${esc(docs[state.module] || "")}</textarea><label class="check"><input id="facts-confirmed" type="checkbox"${disabled()}>Tôi đã kiểm tra tính chính xác và quyền chia sẻ các thông tin này.</label><div class="actions"><button id="save-profile"${disabled()}>Lưu phiên bản hồ sơ</button><button id="preview-profile" class="secondary"${disabled()}>Kiểm tra bằng chứng</button></div><div id="evidence-preview"></div></section><section class="card"><h2>Bằng chứng hiện tại</h2><p class="muted">${esc(state.profile.evidence.facts.length)} mục xác minh · ${esc(state.profile.evidence.weekly_hours)} giờ/tuần · tối thiểu $${esc(state.profile.evidence.minimum_hourly_usd)}/giờ</p>${state.profile.evidence.facts.map((f) => `<div class="status-line"><div>${esc(f.statement)}<small class="muted"> · ${esc(f.source)}</small></div>${badge(f.shareable ? "Có thể chia sẻ" : "Riêng tư")}</div>`).join("")}</section>`;
}
async function profile() {
  const generation = state.route;
  const value = await api("/v1/profile");
  if (generation !== state.route) throw new Error("Navigation changed");
  state.profile = value;
  return profileEditor();
}
function templateEditor() {
  const catalog = state.config.effective.radar.proposals.templates;
  if (!catalog?.items.length)
    return `<section class="card empty"><h2>Chưa có mẫu proposal</h2><p>Thêm mẫu đầu tiên; chỉ lưu khi bạn xác nhận.</p><button id="add-template"${disabled()}>+ Thêm mẫu</button></section>`;
  state.template = Math.min(state.template, catalog.items.length - 1);
  const t = catalog.items[state.template];
  return `<div class="notice">Mẫu chỉ hướng dẫn cách viết; trải nghiệm và thành tích vẫn phải lấy từ hồ sơ đã xác minh.</div><div class="template-grid"><section class="card template-list"><h2>Thư viện mẫu</h2>${catalog.items.map((v, i) => `<button data-template="${i}" class="secondary" aria-pressed="${i === state.template}">${esc(v.name)}</button>`).join("")}<button id="add-template"${disabled()}>+ Thêm mẫu</button></section><section class="card"><h2>${esc(t.name)}</h2><form id="template-form">${[
    ["id", "ID ổn định"],
    ["name", "Tên mẫu"],
    ["use_when", "Dùng khi nào"],
    ["instructions", "Hướng dẫn viết"],
    ["mock_approach", "Cách tiếp cận mẫu"],
    ["question", "Câu hỏi gợi ý"],
  ]
    .map(
      ([k, n]) =>
        `<label for="tpl-${k}">${n}</label>${["instructions", "mock_approach"].includes(k) ? `<textarea id="tpl-${k}" name="${k}" rows="4" required${disabled()}>${esc(t[k])}</textarea>` : `<input id="tpl-${k}" name="${k}" value="${esc(t[k])}" required${disabled()}>`}`,
    )
    .join("")}<div class="grid">${[
    ["max_words", "Số từ tối đa", 60, 300],
    ["max_evidence", "Bằng chứng tối đa", 1, 2],
    ["max_questions", "Câu hỏi tối đa", 0, 2],
  ]
    .map(
      ([k, n, min, max]) =>
        `<div><label for="tpl-${k}">${n}</label><input id="tpl-${k}" name="${k}" type="number" min="${min}" max="${max}" value="${t[k]}" required${disabled()}></div>`,
    )
    .join(
      "",
    )}</div><label class="check"><input id="default-template" type="checkbox" ${catalog.default_id === t.id ? "checked" : ""}${disabled()}>Dùng làm mẫu mặc định</label><div class="actions"><button${disabled()}>Lưu mẫu vào cấu hình</button></div></form></section></div>`;
}
async function templates() {
  const generation = state.route;
  const value = await api("/v1/config");
  if (generation !== state.route) throw new Error("Navigation changed");
  state.config = value;
  return templateEditor();
}
async function activity() {
  const [feedback, notifications, scheduler] = await Promise.all([
    api("/v1/feedback?page_size=50"),
    api("/v1/notifications"),
    api("/v1/scheduler"),
  ]);
  return `<section class="card"><h2>Feedback gần đây</h2><p class="muted">Hiện ${feedback.items.length} / ${feedback.total} sự kiện gần nhất. Bỏ qua job chưa có feedback không tạo tín hiệu âm.</p>${feedback.items.length ? `<div class="table-wrap"><table><thead><tr><th>Job</th><th>Quyết định & lý do</th><th>Thời gian</th></tr></thead><tbody>${feedback.items.map((f) => `<tr><td><a href="#job/${esc(f.job_id)}">${esc(f.job_title)}</a></td><td>${badge(f.action)}<small>${esc(label(f.reason_code))}</small><small>${esc(f.reason || "")}</small></td><td>${esc(date(f.created_at))}</td></tr>`).join("")}</tbody></table></div>` : '<p class="empty">Chưa có feedback.</p>'}</section><div class="grid"><section class="card"><h2>Thông báo</h2>${notifications.providers.map((p) => `<p>${badge(p.instance, p.configured ? "good" : "")} ${esc(p.diagnostic)}</p>`).join("")}${notifications.deliveries.map((d) => `<div class="status-line"><div>${esc(d.title)}<small> · ${esc(d.destination)}</small>${d.error ? `<small class="inline-error">${esc(d.error)}</small>` : ""}</div>${badge(d.status)}</div>`).join("") || '<p class="empty">Chưa có thông báo.</p>'}</section><section class="card"><h2>Lịch sử scan</h2>${scheduler.runs.map((r) => `<div class="status-line"><div>${esc(r.source)} · ${esc(r.origin)}<small> · ${esc(date(r.scheduled_at))}</small>${r.error ? `<small class="inline-error">${esc(r.error)}</small>` : ""}</div>${badge(r.status, r.status === "failed" ? "bad" : "")}</div>`).join("") || '<p class="empty">Chưa có lần chạy.</p>'}</section></div>`;
}
async function settings() {
  const generation = state.route;
  const [config, s, u, p] = await Promise.all([
    api("/v1/config"),
    api("/v1/scheduler"),
    api("/v1/upwork"),
    api("/v1/preferences"),
  ]);
  if (generation !== state.route) throw new Error("Navigation changed");
  state.config = config;
  const q = u.quota;
  const quota = q
    ? `<section class="card"><h2>Quota Upwork MCP</h2><p class="muted">Quota cục bộ của Radar, không phải hạn mức tài khoản Upwork. Không hiển thị token.</p>${line("Phút", `${q.minute_used} / ${q.minute_limit} · reset ${date(q.minute_reset_at)}`)}${line("Giờ", `${q.hour_used} / ${q.hour_limit} · reset ${date(q.hour_reset_at)}`)}${line("Ngày", `${q.day_used} / ${q.day_limit} · reset ${date(q.day_reset_at)}`)}${q.blocked_until ? line("Tạm hoãn đến", date(q.blocked_until)) : ""}</section>`
    : "";
  return `<div class="grid"><section class="card"><h2>Vận hành local</h2>${line("Chế độ", label(s.mode))}${line("Workers", s.active_workers)}${line("Lịch tiếp theo", date(s.next_scan))}<div class="actions"><button id="scan-mock"${disabled()}>Scan mock</button><button id="pause" class="secondary"${disabled()}>${s.mode === "paused" ? "Tiếp tục" : "Tạm dừng"}</button></div><p class="muted"><small>Yêu cầu scan được đưa vào hàng đợi. Thành công khi xếp hàng chưa có nghĩa scan đã hoàn tất.</small></p><label for="work-mode">Chế độ làm việc</label><select id="work-mode">${["employed", "available", "busy", "paused"].map((v) => `<option ${v === s.mode ? "selected" : ""}>${v}</option>`).join("")}</select><button id="save-mode" class="secondary"${disabled()}>Lưu chế độ</button></section><section class="card"><h2>Upwork · Tìm thủ công</h2>${line("Trạng thái", u.enabled ? "Đã bật" : "Đã tắt")}${line("Xác thực", label(u.authentication))}${line("Chế độ chạy", label(u.execution_mode))}${line("Lần tìm gần nhất", date(u.last_successful_search))}${line("Kết quả / mới / trùng", `${u.fetched} / ${u.new_jobs} / ${u.duplicates}`)}${u.last_error_category ? line("Lỗi gần nhất", u.last_error_category) : ""}${u.filters_ignored?.length ? `<p class="inline-error">Bộ lọc bị bỏ qua upstream: ${esc(u.filters_ignored.join(", "))}</p>` : ""}<label for="search-profile">Search profile (để trống dùng mặc định)</label><input id="search-profile" placeholder="Tên profile đã cấu hình"><div class="actions"><button id="scan-upwork" ${!u.enabled ? "disabled" : disabled()}>Tìm job Upwork</button></div><small>Định kỳ chỉ chạy khi execution_mode là opted_in. Login/reauthorization vẫn dùng lệnh Upwork hiện có.</small></section></div>${quota}<section class="card"><h2>Cá nhân hóa ranking</h2><p class="muted">Học từ feedback rõ ràng của bạn; không thay đổi điều kiện bắt buộc.</p>${line("Trạng thái", p.enabled ? "Đã bật" : "Đã tắt")}${line("Số model", p.models.length)}${line("Model hiện tại", p.active_model ? "Đã chọn" : "Chưa kích hoạt")}${p.stale ? badge("Cần rebuild sau feedback mới", "warn") : ""}<details><summary>Chẩn đoán / cấu hình model</summary><pre>${esc(JSON.stringify(p, null, 2))}</pre><p>So sánh, kích hoạt và rollback model qua TUI hiện có. Giao diện web hiện hiển thị dữ liệu, không tự huấn luyện hoặc kích hoạt.</p></details></section><section class="card"><h2>Cấu hình nâng cao</h2><p class="muted">Chỉnh một namespace JSON. Server kiểm tra hợp lệ và lưu revision mới; không ghi đè YAML hoặc secrets.</p><label for="namespace">Namespace</label><select id="namespace"><option>radar</option><option>scoring</option><option>notifications</option></select><label for="config-json">Nội dung</label><textarea id="config-json" class="editor"${disabled()}>${esc(JSON.stringify(config.effective.radar, null, 2))}</textarea><label class="check"><input type="checkbox" id="confirm-config"${disabled()}>Tôi đã xem thay đổi cấu hình trước khi lưu.</label><button id="save-config"${disabled()}>Kiểm tra & lưu cấu hình</button></section>`;
}
const pendingKeys = new Map();
async function mutate(path, body, method) {
  if (!state.write) throw new Error("Phiên này chỉ được xem.");
  const key = path + JSON.stringify(body);
  if ("idempotency_key" in body) {
    if (!pendingKeys.has(key)) pendingKeys.set(key, crypto.randomUUID());
    body.idempotency_key = pendingKeys.get(key);
  }
  const result = await api(path, body, method);
  pendingKeys.delete(key);
  return result;
}
async function saveConfig(namespace, value) {
  const body = {
    expected_revision: state.config.revision,
    radar: null,
    scoring: null,
    notifications: null,
  };
  body[namespace] = value;
  state.config = await mutate("/v1/config", body, "PUT");
  state.dirty = false;
}
async function feedback(action) {
  const body = {
    idempotency_key: null,
    expected_revision: state.job.revision,
    action,
    reason_code: $("#reason-code").value || null,
    reason: $("#reason").value || null,
    confirmed_submission: false,
  };
  if (action === "applied") {
    if (!$("#confirm-applied").checked || !$("#submitted-at").value)
      throw new Error(
        "Cần xác nhận bạn đã tự gửi proposal và chọn thời điểm đã gửi.",
      );
    body.confirmed_submission = true;
    body.submitted_at = new Date($("#submitted-at").value).toISOString();
  }
  await mutate(`/v1/jobs/${state.job.id}/feedback`, body);
  state.dirty = false;
  message("Đã lưu feedback trên server.");
  await route();
}
function bind(view) {
  if (view === "jobs") {
    $("#job-search").onsubmit = (e) => {
      e.preventDefault();
      state.search = $("#search").value.trim();
      state.verdict = $("#verdict").value;
      state.page = 1;
      state.dirty = false;
      route();
    };
    $("#prev").onclick = () => {
      state.page--;
      route();
    };
    $("#next").onclick = () => {
      state.page++;
      route();
    };
  }
  if (view === "job") {
    main
      .querySelectorAll("[data-feedback]")
      .forEach(
        (b) => (b.onclick = () => action(() => feedback(b.dataset.feedback))),
      );
    $("#save-outcome").onclick = () =>
      action(() => feedback($("#outcome").value));
    const start = (kind) =>
      action(async () => {
        if (
          state.dirty &&
          !confirm(
            "Bạn có phần đang sửa chưa lưu. Tiếp tục và tải lại dữ liệu?",
          )
        )
          return;
        const evaluation = state.ai.operations.find(
          (o) => o.kind === "evaluate" && o.approved && !o.stale,
        );
        if (
          !confirm(
            kind === "evaluate"
              ? "Gửi thông tin job và bằng chứng hồ sơ đến AI provider đã cấu hình? Có thể phát sinh phí."
              : "Tạo proposal mới bằng AI? Bản chỉnh sửa cũ vẫn được giữ. Có thể phát sinh phí.",
          )
        )
          return;
        const op = await mutate(
          `/v1/jobs/${state.job.id}/ai/${kind === "evaluate" ? "evaluate" : "proposals"}`,
          {
            idempotency_key: null,
            expected_revision: state.job.revision,
            evaluation_id: kind === "proposal" ? evaluation?.id : null,
            template_id:
              kind === "proposal"
                ? $("#proposal-template").value || null
                : null,
          },
        );
        state.dirty = false;
        message(
          `Đã nhận yêu cầu · ${label(op.status)}. Nhấn Làm mới để kiểm tra kết quả.`,
        );
        await route();
      });
    $("#evaluate").onclick = () => start("evaluate");
    $("#generate").onclick = () => start("proposal");
    const evaluation = state.ai.operations.find((o) => o.kind === "evaluate");
    if ($("#approve-ai"))
      $("#approve-ai").onclick = () =>
        action(async () => {
          await mutate(`/v1/jobs/${state.job.id}/ai/${evaluation.id}/approve`, {
            expected_revision: state.job.revision,
          });
          message("Đã duyệt đánh giá.");
          await route();
        });
    if ($("#cancel-ai"))
      $("#cancel-ai").onclick = () =>
        action(async () => {
          await mutate(
            `/v1/jobs/${state.job.id}/ai/${evaluation.id}/cancel`,
            {},
          );
          message("Đã hủy yêu cầu.");
          await route();
        });
    const saveDraft = (approve) =>
      action(async () => {
        if (
          approve &&
          !confirm(
            "Duyệt nội dung này làm bản cuối? Thao tác không gửi lên Upwork.",
          )
        )
          return;
        const draft = state.ai.drafts[0];
        await mutate(
          `/v1/jobs/${state.job.id}/ai/proposal`,
          {
            expected_revision: draft.revision,
            text: $("#draft-text").value,
            approve,
          },
          "PUT",
        );
        state.dirty = false;
        message(
          approve
            ? "Đã lưu bản cuối được duyệt. Chưa gửi proposal."
            : "Đã lưu chỉnh sửa draft.",
        );
        await route();
      });
    if ($("#save-draft")) $("#save-draft").onclick = () => saveDraft(false);
    if ($("#approve-draft"))
      $("#approve-draft").onclick = () => saveDraft(true);
    if ($("#copy-draft"))
      $("#copy-draft").onclick = () =>
        action(async () => {
          try {
            await navigator.clipboard.writeText($("#draft-text").value);
            message("Đã copy nội dung.");
          } catch {
            $("#draft-text").focus();
            $("#draft-text").select();
            throw new Error(
              "Trình duyệt chưa cho phép clipboard. Nội dung đã được chọn; nhấn Ctrl+C.",
            );
          }
        });
  }
  if (view === "profile") {
    const capture = () => {
      state.profile.profile.documents[state.module] =
        $("#profile-document").value;
    };
    main.querySelectorAll("[data-module]").forEach(
      (b) =>
        (b.onclick = () => {
          capture();
          state.module = b.dataset.module;
          show(profileEditor());
          bind("profile");
        }),
    );
    const request = () => ({
      expected_revision: state.profile.revision,
      documents: state.profile.profile.documents,
      confirmed_facts: $("#facts-confirmed").checked,
    });
    $("#save-profile").onclick = () =>
      action(async () => {
        capture();
        if (!$("#facts-confirmed").checked)
          throw new Error(
            "Hãy xác nhận các thông tin hồ sơ là chính xác trước khi lưu.",
          );
        state.profile = await mutate("/v1/profile", request(), "PUT");
        state.dirty = false;
        show(profileEditor());
        bind("profile");
        message("Đã lưu phiên bản hồ sơ mới.");
      });
    $("#preview-profile").onclick = () =>
      action(async () => {
        capture();
        const evidence = await api("/v1/profile/preview", request());
        $("#evidence-preview").textContent =
          `Hợp lệ: ${evidence.facts.length} bằng chứng, tối thiểu $${evidence.minimum_hourly_usd}/giờ. Chưa lưu thay đổi.`;
      });
  }
  if (view === "templates") {
    const capture = () => {
      const form = $("#template-form");
      if (!form) return;
      const t =
        state.config.effective.radar.proposals.templates.items[state.template];
      new FormData(form).forEach((v, k) => {
        t[k] = k.startsWith("max_") ? Number(v) : v;
      });
    };
    main.querySelectorAll("[data-template]").forEach(
      (b) =>
        (b.onclick = () => {
          capture();
          state.template = Number(b.dataset.template);
          show(templateEditor());
          bind("templates");
        }),
    );
    if ($("#add-template"))
      $("#add-template").onclick = () => {
        capture();
        const catalog = (state.config.effective.radar.proposals.templates ||= {
          default_id: null,
          items: [],
        });
        catalog.items.push({
          id: `template_${Date.now()}`,
          name: "Mẫu mới",
          use_when: "Mô tả loại job phù hợp",
          instructions:
            "Mở đầu bằng vấn đề của khách hàng; nêu cách giải quyết cụ thể và chỉ dùng bằng chứng xác minh.",
          mock_approach:
            "Làm rõ phạm vi, thực hiện phần nhỏ nhất có thể kiểm chứng, bàn giao với kiểm thử.",
          question: "Kết quả nào quan trọng nhất cho lần bàn giao đầu?",
          max_words: 150,
          max_evidence: 1,
          max_questions: 1,
        });
        state.template = catalog.items.length - 1;
        state.dirty = true;
        show(templateEditor());
        bind("templates");
      };
    if ($("#template-form"))
      $("#template-form").onsubmit = (e) => {
        e.preventDefault();
        action(async () => {
          const old =
            state.config.effective.radar.proposals.templates.items[
              state.template
            ].id;
          capture();
          const catalog = state.config.effective.radar.proposals.templates;
          const selected = catalog.items[state.template].id;
          if ($("#default-template").checked) catalog.default_id = selected;
          else if (catalog.default_id === old) catalog.default_id = null;
          await saveConfig("radar", state.config.effective.radar);
          message("Đã lưu thư viện mẫu proposal.");
          show(templateEditor());
          bind("templates");
        });
      };
  }
  if (view === "settings") {
    const scan = (upwork) =>
      action(async () => {
        const path = upwork ? "/v1/upwork/scan" : "/v1/scheduler/scan";
        const body = { idempotency_key: null };
        if (upwork) {
          if (!confirm("Tìm job Upwork thủ công với search profile đã chọn?"))
            return;
          body.profile = $("#search-profile").value.trim() || null;
        }
        const run = await mutate(path, body);
        message(
          `Đã xếp hàng scan · ${label(run.status)}. Xem Lịch sử để biết kết quả.`,
        );
      });
    $("#scan-mock").onclick = () => scan(false);
    $("#scan-upwork").onclick = () => scan(true);
    $("#pause").onclick = () =>
      action(async () => {
        const config = state.config;
        await mutate("/v1/scheduler/mode", {
          idempotency_key: null,
          expected_revision: config.revision,
          paused: config.effective.radar.app.active_mode !== "paused",
        });
        state.dirty = false;
        message("Đã cập nhật trạng thái scheduler.");
        await route();
      });
    $("#save-mode").onclick = () =>
      action(async () => {
        const radar = structuredClone(state.config.effective.radar);
        radar.app.active_mode = $("#work-mode").value;
        await saveConfig("radar", radar);
        message("Đã lưu chế độ.");
        await route();
      });
    $("#namespace").onchange = () => {
      if (
        state.dirty &&
        !confirm("Bỏ phần cấu hình chưa lưu để chuyển namespace?")
      ) {
        $("#namespace").value = state.namespace || "radar";
        return;
      }
      state.namespace = $("#namespace").value;
      $("#config-json").value = JSON.stringify(
        state.config.effective[$("#namespace").value],
        null,
        2,
      );
      state.dirty = false;
    };
    $("#save-config").onclick = () =>
      action(async () => {
        if (!$("#confirm-config").checked)
          throw new Error("Cần xác nhận bạn đã xem thay đổi.");
        let value;
        try {
          value = JSON.parse($("#config-json").value);
        } catch {
          throw new Error("JSON chưa hợp lệ. Thay đổi chưa được gửi.");
        }
        await saveConfig($("#namespace").value, value);
        message("Server đã kiểm tra và lưu cấu hình.");
        await route();
      });
  }
}
async function route() {
  const generation = ++state.route;
  if (!state.connected) {
    login();
    return;
  }
  if (
    state.dirty &&
    !confirm("Có nội dung chưa lưu. Bạn muốn tải trang mới và bỏ thay đổi?")
  ) {
    if (state.currentHash) history.replaceState(null, "", state.currentHash);
    return;
  }
  if (
    state.currentHash &&
    state.currentHash !== (location.hash || "#dashboard")
  )
    message("");
  state.currentHash = location.hash || "#dashboard";
  const [view, id] = (location.hash.slice(1) || "dashboard").split("/");
  $("#page-title").textContent =
    view === "job" ? "Chi tiết job" : titles[view] || "Không tìm thấy";
  document.querySelectorAll("nav a").forEach((a) => {
    if (a.hash === `#${view === "job" ? "jobs" : view}`)
      a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  });
  state.dirty = false;
  main.innerHTML = '<p class="loading" role="status">Đang tải dữ liệu…</p>';
  try {
    const loaders = { dashboard, jobs, profile, templates, activity, settings };
    const html = await (view === "job"
      ? jobDetails(id)
      : loaders[view]
        ? loaders[view]()
        : Promise.resolve(
            '<section class="card"><h2>Không tìm thấy trang</h2><a href="#dashboard">Về Tổng quan</a></section>',
          ));
    if (generation !== state.route) return;
    show(html);
    bind(view);
    if ($("#message").classList.contains("error")) message("");
    $("#connection").textContent = state.write
      ? "Đã kết nối · Chủ sở hữu"
      : "Đã kết nối · Chỉ xem";
  } catch (error) {
    if (generation !== state.route) return;
    main.innerHTML =
      '<section class="card"><h2>Chưa tải được dữ liệu</h2><p>Kiểm tra kết nối backend/Supabase rồi nhấn Làm mới. Không có thay đổi nào được xác nhận.</p><button id="reconnect" class="secondary">Đăng nhập lại</button></section>';
    $("#reconnect").onclick = () => {
      state.connected = false;
      login();
    };
    message(error.message, true);
    $("#connection").textContent = "Cần kiểm tra kết nối";
  }
}
$("#refresh").onclick = () => {
  if (!state.busy) route();
};
document.querySelector("nav").addEventListener("click", (e) => {
  if (state.busy) {
    e.preventDefault();
    message("Đang xử lý thao tác. Vui lòng chờ kết quả.");
  }
});
$("#logout").onclick = () =>
  action(async () => {
    if (state.dirty && !confirm("Đăng xuất và bỏ phần chưa lưu?")) return;
    let revoked = true;
    try {
      await api("/web/logout", {});
    } catch {
      revoked = false;
    }
    state.connected = false;
    state.write = false;
    state.dirty = false;
    state.profile = null;
    state.job = null;
    state.ai = null;
    state.config = null;
    pendingKeys.clear();
    $("#connection").textContent = "Chưa kết nối";
    message(
      revoked
        ? "Đã đăng xuất và thu hồi phiên trình duyệt."
        : "Chưa thể đăng xuất vì mất kết nối. Phiên có thể vẫn còn hiệu lực tối đa 2 giờ; hãy kết nối lại và thử Đăng xuất lần nữa.",
      !revoked,
    );
    login();
  });
window.addEventListener("hashchange", () => route());
window.addEventListener("beforeunload", (e) => {
  if (state.dirty) {
    e.preventDefault();
    e.returnValue = "";
  }
});
(async () => {
  const params = new URLSearchParams(location.hash.slice(1));
  const code = params.get("connect");
  if (code) {
    history.replaceState(null, "", location.pathname + "#dashboard");
    try {
      await connect(code);
    } catch (error) {
      message(error.message, true);
      state.connected = false;
    }
  }
  if (!state.connected) {
    try {
      setSession(await api("/v1/session"));
    } catch {
      state.connected = false;
    }
  }
  await route();
})();
