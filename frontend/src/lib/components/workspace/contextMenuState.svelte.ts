export type ContextMenuItem = {
  id: string;
  label: string;
  icon?: string;
  checked?: boolean;
  disabled?: boolean;
  danger?: boolean;
  dividerBefore?: boolean;
  onSelect: () => void | Promise<void>;
};

type ContextMenuStatus = 'closed' | 'open' | 'closing';

type ContextMenuSnapshot = {
  status: ContextMenuStatus;
  items: ContextMenuItem[];
  ariaLabel: string;
  x: number;
  y: number;
};

const closeAnimationMs = 120;

let closeTimer: ReturnType<typeof setTimeout> | undefined;
let menu = $state<ContextMenuSnapshot>({
  status: 'closed',
  items: [],
  ariaLabel: 'Menu kontekstowe',
  x: 0,
  y: 0
});

export const contextMenu = {
  get status() {
    return menu.status;
  },
  get isVisible() {
    return menu.status !== 'closed';
  },
  get items() {
    return menu.items;
  },
  get ariaLabel() {
    return menu.ariaLabel;
  },
  get x() {
    return menu.x;
  },
  get y() {
    return menu.y;
  }
};

export function openContextMenu(input: {
  items: ContextMenuItem[];
  ariaLabel?: string;
  x: number;
  y: number;
}) {
  const enabledItems = input.items.filter((item) => !item.disabled);

  if (enabledItems.length === 0) {
    closeContextMenu();
    return;
  }

  clearCloseTimer();
  menu = {
    status: 'open',
    items: input.items,
    ariaLabel: input.ariaLabel ?? 'Menu kontekstowe',
    x: input.x,
    y: input.y
  };
}

export function closeContextMenu() {
  if (menu.status === 'closed') {
    return;
  }

  clearCloseTimer();
  menu.status = 'closing';
  closeTimer = setTimeout(() => {
    menu = {
      ...menu,
      status: 'closed',
      items: []
    };
    closeTimer = undefined;
  }, closeAnimationMs);
}

export function positionContextMenu(x: number, y: number) {
  if (menu.status === 'closed') {
    return;
  }

  menu.x = x;
  menu.y = y;
}

function clearCloseTimer() {
  if (closeTimer === undefined) {
    return;
  }

  clearTimeout(closeTimer);
  closeTimer = undefined;
}
