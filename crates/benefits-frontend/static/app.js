import { benefits, defaultBenefitId } from "./benefits-data.js";

const API_BASE_URL = window.BENEFITS_API_URL ?? "http://127.0.0.1:3001";
const STORAGE_PREFIX = "benefits-web";
const screens = [...document.querySelectorAll("[data-screen]")];
const stepItems = [...document.querySelectorAll(".stepper__item")];
const formSteps = [...document.querySelectorAll(".form-step")];
const submitStatus = document.querySelector(".submit-status");
const jsonPreview = document.querySelector("[data-json-preview]");
const benefitItems = [...document.querySelectorAll("[data-benefit-id]")];
const benefitEntryCard = document.querySelector(".benefit-entry-card");
const benefitEntryCardShine = document.querySelector(".benefit-entry-card__shine");
const statusBindings = {
  title: document.querySelector("[data-status-title]"),
  pill: document.querySelector("[data-status-pill]"),
  submittedAt: document.querySelector("[data-status-submitted-at]"),
  review: document.querySelector("[data-status-review]"),
  approval: document.querySelector("[data-status-approval]"),
  payout: document.querySelector("[data-status-payout]"),
  lastRequestCard: document.querySelector("[data-last-request]"),
  lastRequestTitle: document.querySelector("[data-last-request-title]"),
  lastRequestDate: document.querySelector("[data-last-request-date]"),
  lastRequestNumber: document.querySelector("[data-last-request-number]"),
  lastRequestStatus: document.querySelector("[data-last-request-status]"),
};

const detailBindings = {
  name: document.querySelector("[data-detail-name]"),
  category: document.querySelector("[data-detail-category]"),
  title: document.querySelector("[data-detail-title]"),
  description: document.querySelector("[data-detail-description]"),
  limit: document.querySelector("[data-detail-limit]"),
  coverage: document.querySelector("[data-detail-coverage]"),
  period: document.querySelector("[data-detail-period]"),
  approver: document.querySelector("[data-detail-approver]"),
  reviewTime: document.querySelector("[data-detail-review-time]"),
  documents: document.querySelector("[data-detail-documents]"),
  faqPrimary: document.querySelector("[data-detail-faq-primary]"),
  faqSecondary: document.querySelector("[data-detail-faq-secondary]"),
};
const applyBenefitTitle = document.querySelector("[data-apply-benefit-title]");
const validScreens = new Set(screens.map((screen) => screen.dataset.screen));
const validSteps = new Set(stepItems.map((item) => item.dataset.stepTarget));

let activeScreen = loadStoredValue("active-screen", "home");
let activeStep = loadStoredValue("active-step", "1");
let activeBenefitId = loadSelectedBenefitId();
let formCache = loadForms();
let submittedRequest = loadSubmittedRequest();
let benefitEntryShineAnimation = null;
let benefitEntrySlowdownFrame = null;

function storageKey(name) {
  return `${STORAGE_PREFIX}.${name}`;
}

function loadStoredValue(name, fallback) {
  const value = localStorage.getItem(storageKey(name));
  return value || fallback;
}

function loadSelectedBenefitId() {
  const stored = loadStoredValue("selected-benefit-id", defaultBenefitId);
  return stored && benefits[stored] ? stored : defaultBenefitId;
}

function saveSelectedBenefitId() {
  localStorage.setItem(storageKey("selected-benefit-id"), activeBenefitId);
}

function loadForms() {
  try {
    return JSON.parse(localStorage.getItem(storageKey("forms")) ?? "{}");
  } catch {
    return {};
  }
}

function saveForms() {
  localStorage.setItem(storageKey("forms"), JSON.stringify(formCache));
}

function loadSubmittedRequest() {
  try {
    return JSON.parse(localStorage.getItem(storageKey("submitted-request")) ?? "null");
  } catch {
    return null;
  }
}

function saveSubmittedRequest() {
  localStorage.setItem(storageKey("submitted-request"), JSON.stringify(submittedRequest));
}

function currentBenefit() {
  return benefits[activeBenefitId] ?? benefits[defaultBenefitId];
}

function currentForm() {
  return formCache[activeBenefitId] ?? { ...currentBenefit().defaults };
}

function initializeBenefitEntryCardAnimation() {
  if (!benefitEntryCard || !benefitEntryCardShine) {
    return;
  }

  benefitEntryShineAnimation = benefitEntryCardShine.animate(
    [
      { transform: "skewX(-20deg) translateX(0)" },
      { transform: "skewX(-20deg) translateX(420%)" },
    ],
    {
      duration: 2600,
      iterations: Infinity,
      easing: "linear",
    },
  );

  const slowTo = (targetRate) => {
    if (!benefitEntryShineAnimation) {
      return;
    }

    if (benefitEntrySlowdownFrame) {
      cancelAnimationFrame(benefitEntrySlowdownFrame);
    }

    const step = () => {
      if (!benefitEntryShineAnimation) {
        return;
      }

      const currentRate = benefitEntryShineAnimation.playbackRate;
      const nextRate = currentRate + (targetRate - currentRate) * 0.14;

      if (Math.abs(nextRate - targetRate) < 0.02) {
        benefitEntryShineAnimation.playbackRate = targetRate;
        if (targetRate === 0) {
          benefitEntryShineAnimation.pause();
        } else if (benefitEntryShineAnimation.playState !== "running") {
          benefitEntryShineAnimation.play();
        }
        benefitEntrySlowdownFrame = null;
        return;
      }

      if (benefitEntryShineAnimation.playState !== "running") {
        benefitEntryShineAnimation.play();
      }

      benefitEntryShineAnimation.playbackRate = nextRate;
      benefitEntrySlowdownFrame = requestAnimationFrame(step);
    };

    step();
  };

  benefitEntryCard.addEventListener("mouseenter", () => slowTo(0));
  benefitEntryCard.addEventListener("mouseleave", () => {
    if (!benefitEntryShineAnimation) {
      return;
    }

    benefitEntryShineAnimation.play();
    slowTo(1);
  });
}

function triggerPendingDownload() {
  if (!submittedRequest?.autoDownloadPending || !submittedRequest.downloadUrl) {
    return;
  }

  const url = new URL(submittedRequest.downloadUrl, API_BASE_URL);
  const link = document.createElement("a");
  link.href = url.toString();
  link.download = submittedRequest.fileName ?? "statement.pdf";
  link.click();

  submittedRequest.autoDownloadPending = false;
  saveSubmittedRequest();
}

function setScreen(screenName) {
  if (!validScreens.has(screenName)) {
    activeScreen = "home";
  } else {
    activeScreen = screenName;
  }

  localStorage.setItem(storageKey("active-screen"), activeScreen);

  screens.forEach((screen) => {
    screen.classList.toggle("is-active", screen.dataset.screen === activeScreen);
  });

  if (activeScreen === "status") {
    triggerPendingDownload();
  }

  if (activeScreen !== "apply") {
    setStep("1");
  }
}

function setStep(stepNumber) {
  activeStep = validSteps.has(stepNumber) ? stepNumber : "1";
  localStorage.setItem(storageKey("active-step"), activeStep);

  stepItems.forEach((item) => {
    item.classList.toggle("is-active", item.dataset.stepTarget === activeStep);
  });

  formSteps.forEach((step) => {
    step.classList.toggle("is-active", step.dataset.step === activeStep);
  });

  updateJsonPreview();
}

function fieldElements() {
  return [...document.querySelectorAll("[data-statement-field]")];
}

function applyFormValues() {
  const values = currentForm();

  fieldElements().forEach((field) => {
    const key = field.dataset.statementField;
    field.value = values[key] ?? "";
  });

  updateJsonPreview();
}

function syncCurrentFormFromDom() {
  const values = currentForm();

  fieldElements().forEach((field) => {
    values[field.dataset.statementField] = field.value.trim();
  });

  formCache[activeBenefitId] = values;
  saveForms();
}

function selectBenefit(benefitId) {
  if (!benefits[benefitId]) {
    return;
  }

  activeBenefitId = benefitId;
  if (!formCache[benefitId]) {
    formCache[benefitId] = { ...benefits[benefitId].defaults };
    saveForms();
  }

  saveSelectedBenefitId();
  renderBenefitDetail();
  applyFormValues();
}

function renderBenefitDetail() {
  const benefit = currentBenefit();

  benefitItems.forEach((item) => {
    item.classList.toggle("benefit-item--selected", item.dataset.benefitId === benefit.id);
  });

  detailBindings.name.textContent = benefit.name;
  detailBindings.category.textContent = benefit.category;
  detailBindings.title.textContent = benefit.title;
  detailBindings.description.textContent = benefit.description;
  detailBindings.limit.textContent = benefit.limit;
  detailBindings.coverage.textContent = benefit.coverage;
  detailBindings.period.textContent = benefit.period;
  detailBindings.approver.textContent = benefit.approver;
  detailBindings.reviewTime.textContent = benefit.reviewTime;
  detailBindings.faqPrimary.textContent = benefit.faqPrimary;
  detailBindings.faqSecondary.textContent = benefit.faqSecondary;
  if (applyBenefitTitle) {
    applyBenefitTitle.textContent = benefit.title;
  }
  detailBindings.documents.replaceChildren(
    ...benefit.documents.map((item) => {
      const li = document.createElement("li");
      li.textContent = item;
      return li;
    }),
  );
}

function buildStatementPayload() {
  const values = currentForm();

  return {
    statement_number: values.statementNumber,
    recipient: {
      position: values.recipientPosition,
      company_name: values.recipientCompany,
      full_name: values.recipientName,
    },
    applicant: {
      department: values.applicantDepartment,
      full_name: values.applicantName,
    },
    body: values.body,
    date: values.date,
  };
}

function formatSubmittedAt(isoTimestamp) {
  const date = new Date(isoTimestamp);
  return new Intl.DateTimeFormat("ru-RU", {
    day: "numeric",
    month: "long",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function renderStatusScreen() {
  if (!submittedRequest) {
    statusBindings.title.textContent = "Заявок пока нет";
    statusBindings.pill.textContent = "Нет данных";
    statusBindings.submittedAt.textContent = "Нет отправленных заявок";
    statusBindings.review.textContent = "После отправки заявка появится здесь";
    statusBindings.approval.textContent = "Статус будет обновлен после проверки";
    statusBindings.payout.textContent = "Ожидает решения работодателя";
    statusBindings.lastRequestTitle.textContent = "Компенсация обучения";
    statusBindings.lastRequestDate.textContent = "19 апреля 2026";
    statusBindings.lastRequestNumber.textContent = "№ WEB-1760309";
    statusBindings.lastRequestStatus.textContent = "На согласовании";
    statusBindings.lastRequestCard.classList.remove("is-updated");
    return;
  }

  statusBindings.title.textContent = submittedRequest.benefitTitle;
  statusBindings.pill.textContent = `№ ${submittedRequest.statementNumber}`;
  statusBindings.submittedAt.textContent = formatSubmittedAt(submittedRequest.submittedAt);
  statusBindings.review.textContent = `На согласовании у ${submittedRequest.approver}`;
  statusBindings.approval.textContent = `Ожидаемый срок проверки: ${submittedRequest.reviewTime}`;
  statusBindings.payout.textContent = "PDF сформирован и доступен для повторной отправки";
  statusBindings.lastRequestTitle.textContent = submittedRequest.benefitTitle;
  statusBindings.lastRequestDate.textContent = formatSubmittedAt(submittedRequest.submittedAt);
  statusBindings.lastRequestNumber.textContent = `№ ${submittedRequest.statementNumber}`;
  statusBindings.lastRequestStatus.textContent = "Отправлено";
  statusBindings.lastRequestCard.classList.add("is-updated");
}

function updateJsonPreview() {
  if (!jsonPreview) {
    return;
  }

  syncCurrentFormFromDom();
  jsonPreview.textContent = JSON.stringify(buildStatementPayload(), null, 2);
}

function setSubmitStatus(message, tone = "neutral") {
  if (!submitStatus) {
    return;
  }

  submitStatus.textContent = message;
  submitStatus.dataset.tone = tone;
}

async function submitApplication(button) {
  const benefit = currentBenefit();
  const payload = buildStatementPayload();

  button.disabled = true;
  setSubmitStatus("Формируем заявление...", "neutral");

  try {
    const response = await fetch(`${API_BASE_URL}/api/v1/request`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(message || `API returned ${response.status}`);
    }

    const result = await response.json();
    submittedRequest = {
      statementNumber: result.statement_number ?? payload.statement_number,
      benefitId: benefit.id,
      benefitTitle: benefit.title,
      approver: benefit.approver,
      reviewTime: benefit.reviewTime,
      submittedAt: new Date().toISOString(),
      fileName: result.file_name,
      downloadUrl: result.download_url,
      autoDownloadPending: true,
    };
    saveSubmittedRequest();
    renderStatusScreen();
    setSubmitStatus("Заявление сформировано и сохранено на сервере.", "success");
    setScreen("status");
  } catch (error) {
    console.error(error);
    setSubmitStatus("Не удалось сформировать заявление. Проверьте, что benefits-api запущен.", "error");
  } finally {
    button.disabled = false;
  }
}

document.addEventListener("click", (event) => {
  const benefitTrigger = event.target.closest("[data-benefit-id]");
  if (benefitTrigger) {
    selectBenefit(benefitTrigger.dataset.benefitId);
    setScreen("detail");
    return;
  }

  const screenTrigger = event.target.closest("[data-screen-target]");
  if (screenTrigger) {
    setScreen(screenTrigger.dataset.screenTarget);
    return;
  }

  const stepTrigger = event.target.closest("[data-step-target]");
  if (stepTrigger) {
    setStep(stepTrigger.dataset.stepTarget);
    return;
  }

  const submitTrigger = event.target.closest("[data-submit-application]");
  if (submitTrigger) {
    submitApplication(submitTrigger);
  }
});

document.addEventListener("input", (event) => {
  if (event.target.closest("[data-statement-field]")) {
    updateJsonPreview();
  }
});

renderBenefitDetail();
renderStatusScreen();
applyFormValues();
setScreen(activeScreen);
setStep(activeStep);
initializeBenefitEntryCardAnimation();
