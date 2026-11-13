import {
  LOCKED_NAV_IDS,
  MENU_CATEGORIES,
  type NavCategoryDef,
  type NavItemDef,
  catalogCategoryById,
  catalogItemById,
} from "@/lib/nav-catalog";

export type NavPreferences = {
  version: 1;
  hidden_ids: string[];
  category_order: string[];
  item_order: Record<string, string[]>;
};

export const emptyNavPreferences = (): NavPreferences => ({
  version: 1,
  hidden_ids: [],
  category_order: [],
  item_order: {},
});

const ID_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
const MAX_IDS = 64;

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const out: string[] = [];
  for (const entry of value) {
    if (typeof entry !== "string") continue;
    const id = entry.trim();
    if (!ID_RE.test(id)) continue;
    if (!out.includes(id)) out.push(id);
    if (out.length >= MAX_IDS) break;
  }
  return out;
}

function knownCategoryIds(): string[] {
  return MENU_CATEGORIES.map((category) => category.id);
}

function knownItemIds(categoryId: string): string[] {
  return catalogCategoryById(categoryId)?.items.map((item) => item.id) ?? [];
}

function mergeOrder(preferred: string[], known: string[]): string[] {
  const knownSet = new Set(known);
  const ordered = preferred.filter((id) => knownSet.has(id));
  for (const id of known) {
    if (!ordered.includes(id)) ordered.push(id);
  }
  return ordered;
}

export function sanitizeNavPreferences(raw: unknown): NavPreferences {
  const source =
    raw && typeof raw === "object" ? (raw as Record<string, unknown>) : {};
  const hidden = asStringArray(source.hidden_ids).filter((id) => {
    const item = catalogItemById(id);
    return Boolean(item) && !item?.locked && !LOCKED_NAV_IDS.includes(id);
  });
  const categoryOrder = mergeOrder(
    asStringArray(source.category_order),
    knownCategoryIds(),
  );
  const rawItemOrder =
    source.item_order && typeof source.item_order === "object"
      ? (source.item_order as Record<string, unknown>)
      : {};
  const itemOrder: Record<string, string[]> = {};
  for (const category of MENU_CATEGORIES) {
    itemOrder[category.id] = mergeOrder(
      asStringArray(rawItemOrder[category.id]),
      knownItemIds(category.id),
    );
  }
  return {
    version: 1,
    hidden_ids: hidden,
    category_order: categoryOrder,
    item_order: itemOrder,
  };
}

export function applyNavPreferences(
  prefs: NavPreferences,
  options: { isAdmin: boolean },
): NavCategoryDef[] {
  const sanitized = sanitizeNavPreferences(prefs);
  const hidden = new Set(sanitized.hidden_ids);
  return sanitized.category_order
    .map((categoryId) => catalogCategoryById(categoryId))
    .filter((category): category is NavCategoryDef => Boolean(category))
    .map((category) => {
      const order = sanitized.item_order[category.id] ?? [];
      const items = order
        .map((itemId) => catalogItemById(itemId))
        .filter((item): item is NavItemDef => Boolean(item))
        .filter((item) => item && category.items.some((owned) => owned.id === item.id))
        .filter((item) => !hidden.has(item.id))
        .filter((item) => !item.adminOnly || options.isAdmin);
      return { ...category, items };
    })
    .filter((category) => category.items.length > 0);
}

export function toggleNavItemHidden(
  prefs: NavPreferences,
  itemId: string,
  hidden: boolean,
): NavPreferences {
  const next = sanitizeNavPreferences(prefs);
  const item = catalogItemById(itemId);
  if (!item || item.locked) return next;
  const set = new Set(next.hidden_ids);
  if (hidden) set.add(itemId);
  else set.delete(itemId);
  next.hidden_ids = Array.from(set);
  return next;
}

function moveInList(list: string[], id: string, delta: number): string[] {
  const index = list.indexOf(id);
  if (index < 0) return list;
  const target = index + delta;
  if (target < 0 || target >= list.length) return list;
  const copy = [...list];
  const [entry] = copy.splice(index, 1);
  copy.splice(target, 0, entry);
  return copy;
}

export function moveNavCategory(
  prefs: NavPreferences,
  categoryId: string,
  delta: number,
): NavPreferences {
  const next = sanitizeNavPreferences(prefs);
  next.category_order = moveInList(next.category_order, categoryId, delta);
  return next;
}

export function moveNavItem(
  prefs: NavPreferences,
  categoryId: string,
  itemId: string,
  delta: number,
): NavPreferences {
  const next = sanitizeNavPreferences(prefs);
  const current = next.item_order[categoryId];
  if (!current) return next;
  next.item_order = {
    ...next.item_order,
    [categoryId]: moveInList(current, itemId, delta),
  };
  return next;
}
