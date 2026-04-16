"use strict";

function validateDocsManifest(manifest) {
  const errors = [];

  if (!manifest || typeof manifest !== "object") {
    return ["manifest must be an object"];
  }

  if (manifest.schema_version !== 1) errors.push("schema_version must be 1");
  if (!manifest.project || typeof manifest.project.id !== "string") errors.push("project.id is required");
  if (!Array.isArray(manifest.docs) || manifest.docs.length === 0) errors.push("docs must be a non-empty array");

  for (const doc of manifest.docs || []) {
    validateDoc(errors, doc);
  }

  return errors;
}

function validateDoc(errors, doc) {
  requireString(errors, doc.id, "doc.id");
  requireString(errors, doc.title, `${doc.id}.title`);
  requireString(errors, doc.summary, `${doc.id}.summary`);
  requireString(errors, doc.content, `${doc.id}.content`);
  requireStringArray(errors, doc.tags, `${doc.id}.tags`);
  requireStringArray(errors, doc.use_cases, `${doc.id}.use_cases`);
  validateExamples(errors, doc.examples, `${doc.id}.examples`);

  if (!Array.isArray(doc.sections) || doc.sections.length === 0) {
    errors.push(`${doc.id}.sections must be a non-empty array`);
    return;
  }

  for (const section of doc.sections) {
    requireString(errors, section.id, `${doc.id}.sections[].id`);
    requireString(errors, section.title, `${section.id}.title`);
    requireString(errors, section.summary, `${section.id}.summary`);
    requireString(errors, section.content, `${section.id}.content`);
    if (!Number.isInteger(section.level)) errors.push(`${section.id}.level must be an integer`);
    validateExamples(errors, section.examples, `${section.id}.examples`);
  }
}

function validateExamples(errors, examples, field) {
  if (!examples || typeof examples !== "object") {
    errors.push(`${field} is required`);
    return;
  }

  validateExampleList(errors, examples.html, `${field}.html`);
  validateExampleList(errors, examples.json, `${field}.json`);
}

function validateExampleList(errors, examples, field) {
  if (!Array.isArray(examples)) {
    errors.push(`${field} must be an array`);
    return;
  }

  for (const example of examples) {
    requireString(errors, example.title, `${field}[].title`);
    requireString(errors, example.description, `${field}[].description`);
    requireString(errors, example.code, `${field}[].code`);
  }
}

function requireString(errors, value, field) {
  if (typeof value !== "string" || value.trim() === "") {
    errors.push(`${field} must be a non-empty string`);
  }
}

function requireStringArray(errors, value, field) {
  if (!Array.isArray(value)) {
    errors.push(`${field} must be an array`);
    return;
  }

  for (const item of value) {
    if (typeof item !== "string" || item.trim() === "") {
      errors.push(`${field} must contain only non-empty strings`);
      return;
    }
  }
}

module.exports = {
  validateDocsManifest,
};
