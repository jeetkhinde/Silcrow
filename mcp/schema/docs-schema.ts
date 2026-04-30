export interface SilcrowDocsManifest {
  schema_version: 1;
  project: {
    id: string;
    name: string;
    version: string;
  };
  docs: SilcrowDoc[];
  source?: {
    type: string;
    path: string;
    note?: string;
  };
}

export interface SilcrowDoc {
  id: string;
  title: string;
  summary: string;
  content: string;
  sections: SilcrowDocSection[];
  examples: SilcrowExamples;
  tags: string[];
  use_cases: string[];
}

export interface SilcrowDocSection {
  id: string;
  title: string;
  level: number;
  summary: string;
  content: string;
  examples: SilcrowExamples;
}

export interface SilcrowExamples {
  html: SilcrowExample[];
  json: SilcrowExample[];
}

export interface SilcrowExample {
  title: string;
  description: string;
  code: string;
}
