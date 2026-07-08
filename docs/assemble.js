const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const UNITS_DIR = path.join(ROOT, 'docs', 'units');
const TARGETS_DIR = path.join(ROOT, 'docs', 'targets');

const errors = [];
const warnings = [];

function error(msg) {
  errors.push(msg);
}

function warn(msg) {
  warnings.push(msg);
}

function unitsDirExists() {
  try {
    return fs.statSync(UNITS_DIR).isDirectory();
  } catch {
    return false;
  }
}

function targetsDirExists() {
  try {
    return fs.statSync(TARGETS_DIR).isDirectory();
  } catch {
    return false;
  }
}

function parseUnit(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const lines = content.split('\n');

  const firstSep = lines.findIndex(l => l.trim() === '---');
  if (firstSep === -1) {
    error(`unit "${path.basename(filePath)}" (${filePath}): no frontmatter delimiters found`);
    return null;
  }

  const secondSep = lines.findIndex((l, i) => i > firstSep && l.trim() === '---');
  if (secondSep === -1) {
    error(`unit "${path.basename(filePath)}" (${filePath}): no closing frontmatter delimiter found`);
    return null;
  }

  const rawFrontmatter = lines.slice(firstSep + 1, secondSep).join('\n');
  let frontmatter;
  try {
    frontmatter = JSON.parse(rawFrontmatter);
  } catch (e) {
    error(`unit "${path.basename(filePath)}" (${filePath}): frontmatter is not valid JSON — ${e.message}`);
    return null;
  }

  const bodyLines = lines.slice(secondSep + 1);
  let body = bodyLines.join('\n').trim();

  if (!frontmatter.id || typeof frontmatter.id !== 'string' || frontmatter.id.trim() === '') {
    error(`unit "${path.basename(filePath)}" (${filePath}): missing required field "id"`);
  } else if (!/^[a-z0-9-]+$/.test(frontmatter.id)) {
    error(`unit "${path.basename(filePath)}" (${filePath}): id "${frontmatter.id}" does not match required pattern [a-z0-9-]+`);
  }

  if (!frontmatter.title || typeof frontmatter.title !== 'string' || frontmatter.title.trim() === '') {
    error(`unit "${path.basename(filePath)}" (${filePath}): missing required field "title"`);
  }

  if (body) {
    const firstHeadingLine = bodyLines.find(l => l.trim().startsWith('#'));
    if (firstHeadingLine && /^#\s/.test(firstHeadingLine.trim())) {
      error(`unit "${path.basename(filePath)}" (${filePath}): body starts with h1 heading ("# "). Units must use h2+ (##).`);
    }
  }

  return {
    id: frontmatter.id,
    title: frontmatter.title,
    body: body || '',
    filePath: filePath
  };
}

function readUnits() {
  const units = {};
  const files = fs.readdirSync(UNITS_DIR).filter(f => f.endsWith('.md'));

  for (const file of files) {
    const filePath = path.join(UNITS_DIR, file);
    const unit = parseUnit(filePath);
    if (unit) {
      if (units[unit.id]) {
        error(`duplicate unit id "${unit.id}" found in ${units[unit.id].filePath} and ${filePath}`);
      } else {
        units[unit.id] = unit;
      }
    }
  }

  return units;
}

function readTargets() {
  const targets = [];
  const files = fs.readdirSync(TARGETS_DIR).filter(f => f.endsWith('.json'));

  for (const file of files) {
    const filePath = path.join(TARGETS_DIR, file);
    let config;
    try {
      config = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
    } catch (e) {
      error(`${filePath}: invalid JSON — ${e.message}`);
      continue;
    }

    const requiredFields = ['output', 'title', 'units', 'footer'];
    for (const field of requiredFields) {
      if (!(field in config)) {
        error(`${filePath}: missing required field "${field}"`);
      }
    }

    if (!Array.isArray(config.units)) {
      error(`${filePath}: "units" must be an array`);
    } else {
      const seen = new Set();
      for (const id of config.units) {
        if (seen.has(id)) {
          error(`${filePath}: duplicate unit id "${id}" in units array`);
        }
        seen.add(id);
      }
    }

    targets.push({ config, filePath });
  }

  return targets;
}

function validateTargetUnits(targets, units) {
  for (const { config, filePath } of targets) {
    if (!Array.isArray(config.units)) continue;
    for (const id of config.units) {
      if (!units[id]) {
        error(`unit "${id}" listed in ${filePath} but no unit with that id exists in docs/units/`);
      }
    }
  }
}

function validateOutputPaths(targets) {
  for (const { config } of targets) {
    if (!config.output) continue;
    const outputPath = path.resolve(ROOT, config.output);
    const parentDir = path.dirname(outputPath);
    try {
      if (!fs.statSync(parentDir).isDirectory()) {
        error(`output path "${config.output}": parent directory "${path.relative(ROOT, parentDir)}" does not exist`);
      }
    } catch {
      error(`output path "${config.output}": parent directory "${path.relative(ROOT, parentDir)}" does not exist`);
    }
  }
}

function findOrphans(units, targets) {
  const referenced = new Set();
  for (const { config } of targets) {
    if (Array.isArray(config.units)) {
      for (const id of config.units) {
        referenced.add(id);
      }
    }
  }
  for (const id of Object.keys(units)) {
    if (!referenced.has(id)) {
      warn(`unit "${id}" (${units[id].filePath}) is not referenced by any target`);
    }
  }
}

function assemble() {
  const unitsDirOk = unitsDirExists();
  const targetsDirOk = targetsDirExists();

  if (!unitsDirOk) {
    error(`docs/units/ directory not found at ${UNITS_DIR}`);
  }
  if (!targetsDirOk) {
    error(`docs/targets/ directory not found at ${TARGETS_DIR}`);
  }

  const units = unitsDirOk ? readUnits() : {};
  const targets = targetsDirOk ? readTargets() : [];

  if (unitsDirOk && targetsDirOk) {
    validateTargetUnits(targets, units);
    validateOutputPaths(targets);
    findOrphans(units, targets);
  }

  if (errors.length > 0) {
    for (const err of errors) {
      process.stderr.write(`error: ${err}\n`);
    }
    for (const w of warnings) {
      process.stderr.write(`warn: ${w}\n`);
    }
    process.exit(1);
  }

  for (const { config } of targets) {
    const title = config.title || '';
    const footer = config.footer || '';
    const unitBodies = (config.units || [])
      .filter(id => units[id])
      .map(id => units[id].body)
      .filter(body => body.length > 0);

    const parts = [title];
    if (title && unitBodies.length > 0) {
      const titleEndsNewline = title.endsWith('\n');
      if (!titleEndsNewline) {
        parts.push('\n');
      }
    }
    parts.push(unitBodies.join('\n\n'));
    if (unitBodies.length > 0 && footer) {
      parts.push('\n');
    }
    parts.push(footer);

    const output = parts.join('').replace(/\n{3,}/g, '\n\n');

    const outputPath = path.resolve(ROOT, config.output);
    fs.writeFileSync(outputPath, output, 'utf-8');
    process.stdout.write(`wrote: ${config.output}\n`);
  }

  for (const w of warnings) {
    process.stderr.write(`warn: ${w}\n`);
  }

  process.exit(0);
}

assemble();
