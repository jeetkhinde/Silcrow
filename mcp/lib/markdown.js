"use strict";

const { normalizeWhitespace, stripMarkdown } = require("./text");

function parseMarkdown(markdown) {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const blocks = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];
    const heading = line.match(/^(#{1,6})\s+(.+?)\s*$/);

    if (heading) {
      blocks.push({
        type: "heading",
        level: heading[1].length,
        text: stripMarkdown(heading[2]),
        raw: line,
      });
      index += 1;
      continue;
    }

    const fence = line.match(/^```(\S*)\s*$/);
    if (fence) {
      const language = (fence[1] || "text").toLowerCase();
      const code = [];
      index += 1;

      while (index < lines.length && !lines[index].startsWith("```")) {
        code.push(lines[index]);
        index += 1;
      }

      blocks.push({
        type: "code",
        language,
        code: code.join("\n").trim(),
      });

      index += 1;
      continue;
    }

    if (line.trim().startsWith("|") && lines[index + 1] && /^\s*\|?[\s:-]+\|/.test(lines[index + 1])) {
      const tableLines = [];

      while (index < lines.length && lines[index].trim().startsWith("|")) {
        tableLines.push(lines[index]);
        index += 1;
      }

      blocks.push({
        type: "paragraph",
        text: tableLines.map(stripMarkdown).join("\n"),
        raw: tableLines.join("\n"),
      });
      continue;
    }

    const paragraph = [];

    while (
      index < lines.length &&
      !/^(#{1,6})\s+/.test(lines[index]) &&
      !/^```/.test(lines[index]) &&
      !(lines[index].trim().startsWith("|") && lines[index + 1] && /^\s*\|?[\s:-]+\|/.test(lines[index + 1]))
    ) {
      paragraph.push(lines[index]);
      index += 1;
    }

    const text = normalizeWhitespace(paragraph.map(stripMarkdown).join("\n"));
    if (text) {
      blocks.push({
        type: "paragraph",
        text,
        raw: paragraph.join("\n"),
      });
    }
  }

  return blocks;
}

module.exports = {
  parseMarkdown,
};
