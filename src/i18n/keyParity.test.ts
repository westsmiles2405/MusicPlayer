import { describe, expect, it } from "vitest";
import zh from "./locales/zh.json";
import en from "./locales/en.json";

function flattenKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  return Object.entries(obj).flatMap(([key, value]) => {
    const next = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      return flattenKeys(value as Record<string, unknown>, next);
    }
    return [next];
  });
}

describe("i18n key parity", () => {
  it("keeps zh and en key structures identical", () => {
    expect(flattenKeys(en).sort()).toEqual(flattenKeys(zh).sort());
  });
});
