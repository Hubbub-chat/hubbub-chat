import type { Agent } from '../types';

/**
 * Agent id from a display name: diacritics removed ("Chuyên gia" -> "chuyen_gia"),
 * never empty, and never equal to an existing id (upsert would overwrite that agent).
 */
export function slugifyAgentName(name: string, existingIds: string[]): string {
  const base =
    name
      .normalize('NFD')
      .replace(/[̀-ͯ]/g, '')
      .replace(/đ/g, 'd')
      .replace(/Đ/g, 'D')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '_')
      .replace(/^_+|_+$/g, '')
      .slice(0, 24)
      .replace(/_+$/, '') || 'agent';

  let slug = base;
  for (let n = 2; existingIds.includes(slug); n++) {
    slug = `${base}_${n}`;
  }
  return slug;
}

/** Least privilege that still lets each selected tool pass the backend policy check. */
export function permissionsForTools(tools: string[]): Agent['permissions'] {
  const has = (t: string) => tools.includes(t);
  const fsRead = has('fs_read') || has('fs_list')
    ? ['**']
    : has('report_read') || has('report_list')
      ? ['reports/**']
      : [];
  return {
    fs_read: fsRead,
    fs_write: has('report_write') ? ['reports/**'] : [],
    network: has('web_fetch') ? 'open' : has('web_search') ? 'search_only' : 'none',
  };
}
