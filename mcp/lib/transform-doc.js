"use strict";

const path = require("path");
const { parseMarkdown } = require("./markdown");
const { firstSentence, normalizeWhitespace, slugify, stripMarkdown, uniqueSlug } = require("./text");

const TOPIC_TAGS = {
  attributes: ["attributes", "html", "binding", "navigation", "live"],
  events: ["events", "custom-events", "lifecycle"],
  "http-headers": ["headers", "http", "server-driven-ui"],
  "javascript-api": ["javascript-api", "window.Silcrow", "programmatic-api"],
  live: ["live", "sse", "websocket", "real-time"],
  navigator: ["navigator", "routing", "fetch", "history"],
  optimistic: ["optimistic", "snapshot", "revert"],
  runtime: ["runtime", "patch", "data-binding"],
};

function transformMarkdownDoc({ fileName, markdown }) {
  const docId = slugify(path.basename(fileName, path.extname(fileName)));
  const blocks = parseMarkdown(markdown);
  const titleBlock = blocks.find((block) => block.type === "heading" && block.level === 1);
  const title = titleBlock ? titleBlock.text : titleFromFile(fileName);
  const sectionSlugs = new Set();
  const sections = [];
  let currentSection = null;
  const intro = [];

  for (const block of blocks) {
    if (block.type === "heading" && block.level === 1) continue;

    if (block.type === "heading" && block.level >= 2 && block.level <= 3) {
      currentSection = {
        id: `${docId}/${uniqueSlug(block.text, sectionSlugs)}`,
        title: block.text,
        level: block.level,
        summary: "",
        contentParts: [],
        examples: emptyExamples(),
      };
      sections.push(currentSection);
      continue;
    }

    if (block.type === "heading" && block.level > 3) {
      if (currentSection) currentSection.contentParts.push(block.text);
      continue;
    }

    if (!currentSection) {
      if (block.type === "paragraph" && !isSourceModuleLine(block.text)) {
        intro.push(block.text);
      }
      continue;
    }

    if (block.type === "code") {
      addExample(currentSection.examples, block);
      continue;
    }

    if (block.type === "paragraph" && !isSourceModuleLine(block.text)) {
      currentSection.contentParts.push(block.text);
    }
  }

  const examples = collectExamples(sections);
  const content = buildDocContent(intro, sections);
  const summary = firstSentence(intro.join(" ")) || firstSentence(content);
  if (sections.length === 0 && content) {
    sections.push({
      id: `${docId}/overview`,
      title: "Overview",
      level: 2,
      summary: "",
      contentParts: [content],
      examples: emptyExamples(),
    });
  }
  hydrateEmptySections(sections);
  const tags = collectTags({ docId, title, sections, examples });
  const use_cases = collectUseCases(sections);

  return {
    id: docId,
    title,
    summary,
    content,
    sections: sections.map((section) => ({
      id: section.id,
      title: section.title,
      level: section.level,
      summary: firstSentence(section.contentParts.join(" ")),
      content: normalizeWhitespace(section.contentParts.join("\n")),
      examples: section.examples,
    })),
    examples,
    tags,
    use_cases,
  };
}

function hydrateEmptySections(sections) {
  for (let index = 0; index < sections.length; index += 1) {
    const section = sections[index];
    if (normalizeWhitespace(section.contentParts.join("\n"))) continue;

    const descendants = [];
    for (let cursor = index + 1; cursor < sections.length; cursor += 1) {
      const candidate = sections[cursor];
      if (candidate.level <= section.level) break;

      const content = normalizeWhitespace([candidate.title, ...candidate.contentParts].join("\n"));
      if (content) descendants.push(content);
    }

    if (descendants.length > 0) {
      section.contentParts.push(descendants.join("\n"));
    } else {
      section.contentParts.push(`${section.title}.`);
    }
  }
}

function titleFromFile(fileName) {
  return path
    .basename(fileName, path.extname(fileName))
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function emptyExamples() {
  return {
    html: [],
    json: [],
  };
}

function addExample(examples, block) {
  const kind = exampleKind(block);
  if (!kind) return;

  examples[kind].push({
    title: `${kind.toUpperCase()} example`,
    description: "Example extracted from the Silcrow markdown documentation.",
    code: block.code,
  });
}

function exampleKind(block) {
  if (block.language === "html") return "html";
  if (block.language === "json") return "json";

  if (block.language === "javascript" && looksLikeJsonObject(block.code)) return "json";
  if (block.language === "text" && looksLikeHtml(block.code)) return "html";
  if (block.language === "text" && looksLikeJsonObject(block.code)) return "json";

  return null;
}

function looksLikeHtml(code) {
  return /^\s*</.test(code) && />\s*$/.test(code);
}

function looksLikeJsonObject(code) {
  const trimmed = code.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return false;

  try {
    JSON.parse(trimmed);
    return true;
  } catch (_error) {
    return false;
  }
}

function collectExamples(sections) {
  const examples = emptyExamples();

  for (const section of sections) {
    examples.html.push(...section.examples.html);
    examples.json.push(...section.examples.json);
  }

  return examples;
}

function buildDocContent(intro, sections) {
  const parts = [...intro];

  for (const section of sections) {
    parts.push(section.title);
    parts.push(...section.contentParts);
  }

  return normalizeWhitespace(parts.join("\n"));
}

function collectTags({ docId, title, sections, examples }) {
  const tags = new Set([docId, ...TOPIC_TAGS[docId] || []]);

  addTokens(tags, title);

  for (const section of sections) {
    addTokens(tags, section.title);
    for (const part of section.contentParts) addInlineCodeTags(tags, part);
  }

  for (const example of [...examples.html, ...examples.json]) {
    addInlineCodeTags(tags, example.code);
  }

  return Array.from(tags)
    .map((tag) => tag.trim())
    .filter(Boolean)
    .sort((a, b) => a.localeCompare(b));
}

function addTokens(tags, value) {
  for (const token of stripMarkdown(value).split(/\s+/)) {
    const normalized = slugify(token);
    if (normalized.length > 2) tags.add(normalized);
  }
}

function addInlineCodeTags(tags, value) {
  const inlineCodePattern = /`([^`]+)`/g;
  let match;

  while ((match = inlineCodePattern.exec(value)) !== null) {
    tags.add(match[1]);
  }

  for (const token of String(value).match(/s-[a-z-]+|silcrow:[a-z:-]+|Silcrow\.[A-Za-z]+|:[a-z-]+/g) || []) {
    tags.add(token);
  }
}

function collectUseCases(sections) {
  const cases = [];

  for (const section of sections) {
    for (const part of section.contentParts) {
      const why = extractLabeledText(part, "Why to use");
      const when = extractLabeledText(part, "When to use");

      if (why) cases.push(why);
      if (when) cases.push(when);
    }
  }

  return unique(cases).slice(0, 20);
}

function extractLabeledText(value, label) {
  const pattern = new RegExp(`${label}:\\s*(.+?)(?=\\n[A-Z][^\\n:]{1,40}:|$)`, "i");
  const match = value.match(pattern);
  return match ? stripMarkdown(match[1]) : "";
}

function unique(values) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
}

function isSourceModuleLine(value) {
  return /source module\(s\)/i.test(value);
}

module.exports = {
  transformMarkdownDoc,
};
