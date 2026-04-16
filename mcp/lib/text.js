"use strict";

function slugify(value) {
  return String(value)
    .trim()
    .toLowerCase()
    .replace(/[`*_~[\]()]/g, "")
    .replace(/&/g, " and ")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function uniqueSlug(value, used) {
  const base = slugify(value) || "section";
  let candidate = base;
  let suffix = 2;

  while (used.has(candidate)) {
    candidate = `${base}-${suffix}`;
    suffix += 1;
  }

  used.add(candidate);
  return candidate;
}

function stripMarkdown(value) {
  return String(value)
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^>\s?/gm, "")
    .replace(/[*_~]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function firstSentence(value) {
  const text = stripMarkdown(value);
  if (!text) return "";

  const match = text.match(/^(.+?[.!?])(\s|$)/);
  return match ? match[1] : text.slice(0, 240);
}

function normalizeWhitespace(value) {
  return String(value)
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .join("\n")
    .trim();
}

module.exports = {
  firstSentence,
  normalizeWhitespace,
  slugify,
  stripMarkdown,
  uniqueSlug,
};
