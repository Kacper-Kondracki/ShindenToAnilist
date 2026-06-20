export type NotificationTone = 'info' | 'success' | 'warning' | 'error';

export type AppNotification = {
  id: number;
  tone: NotificationTone;
  title: string;
  message: string;
};

export type NotificationInput = {
  tone?: NotificationTone;
  title: string;
  message: string;
  timeoutMs?: number;
};

export type NotificationControllerOptions = {
  maxVisibleNotifications?: number;
  maxBufferedNotifications?: number;
};

export type NotificationController = ReturnType<
  typeof createNotificationController
>;

const defaultTimeoutMs = 10000;
const defaultMaxVisibleNotifications = 4;
const defaultMaxBufferedNotifications = 8;
const visibleNotificationTones = new Set<NotificationTone>([
  'info',
  'warning',
  'error'
]);

type StoredNotification = AppNotification & {
  timeoutMs: number;
};

export function createNotificationController({
  maxVisibleNotifications = defaultMaxVisibleNotifications,
  maxBufferedNotifications = defaultMaxBufferedNotifications
}: NotificationControllerOptions = {}) {
  const visibleLimit = Math.max(1, maxVisibleNotifications);
  const bufferLimit = Math.max(0, maxBufferedNotifications);

  let notifications = $state<StoredNotification[]>([]);
  let bufferedNotifications = $state<StoredNotification[]>([]);
  let exitingNotificationIds = $state<number[]>([]);
  let nextNotificationId = 0;
  const timeoutIds = new Map<number, number>();

  function notify({
    tone = 'info',
    title,
    message,
    timeoutMs = defaultTimeoutMs
  }: NotificationInput) {
    const id = ++nextNotificationId;
    if (!visibleNotificationTones.has(tone)) {
      return id;
    }

    const notification: StoredNotification = {
      id,
      tone,
      title,
      message,
      timeoutMs
    };

    if (reservedVisibleCount() < visibleLimit) {
      showNotification(notification);
    } else {
      bufferNotification(notification);
    }

    return id;
  }

  function dismiss(id: number) {
    clearNotificationTimeout(id);
    if (exitingNotificationIds.includes(id)) {
      return;
    }

    const nextNotifications = notifications.filter(
      (notification) => notification.id !== id
    );

    if (nextNotifications.length === notifications.length) {
      bufferedNotifications = bufferedNotifications.filter(
        (notification) => notification.id !== id
      );
      return;
    }

    exitingNotificationIds = [...exitingNotificationIds, id];
    notifications = nextNotifications;
  }

  function completeDismiss(id: number) {
    if (!exitingNotificationIds.includes(id)) {
      return;
    }

    exitingNotificationIds = exitingNotificationIds.filter(
      (notificationId) => notificationId !== id
    );
    promoteBufferedNotifications();
  }

  function showNotification(notification: StoredNotification) {
    notifications = [...notifications, notification];
    startNotificationTimeout(notification);
  }

  function bufferNotification(notification: StoredNotification) {
    if (bufferLimit === 0) {
      return;
    }

    bufferedNotifications =
      bufferedNotifications.length < bufferLimit
        ? [...bufferedNotifications, notification]
        : [...bufferedNotifications.slice(1), notification];
  }

  function promoteBufferedNotifications() {
    while (
      reservedVisibleCount() < visibleLimit &&
      bufferedNotifications.length > 0
    ) {
      const [nextNotification, ...remainingNotifications] =
        bufferedNotifications;
      bufferedNotifications = remainingNotifications;

      if (nextNotification !== undefined) {
        showNotification(nextNotification);
      }
    }
  }

  function reservedVisibleCount() {
    return notifications.length + exitingNotificationIds.length;
  }

  function startNotificationTimeout(notification: StoredNotification) {
    if (notification.timeoutMs <= 0) {
      return;
    }

    timeoutIds.set(
      notification.id,
      window.setTimeout(() => dismiss(notification.id), notification.timeoutMs)
    );
  }

  function clearNotificationTimeout(id: number) {
    const timeoutId = timeoutIds.get(id);
    if (timeoutId !== undefined) {
      window.clearTimeout(timeoutId);
      timeoutIds.delete(id);
    }
  }

  function clear() {
    for (const timeoutId of timeoutIds.values()) {
      window.clearTimeout(timeoutId);
    }

    timeoutIds.clear();
    notifications = [];
    bufferedNotifications = [];
    exitingNotificationIds = [];
  }

  function notifyWithTone(
    tone: NotificationTone,
    title: string,
    message: string,
    timeoutMs?: number
  ) {
    return timeoutMs === undefined
      ? notify({ tone, title, message })
      : notify({ tone, title, message, timeoutMs });
  }

  return {
    get notifications() {
      return notifications;
    },
    get bufferedCount() {
      return bufferedNotifications.length;
    },
    notify,
    info(title: string, message: string, timeoutMs?: number) {
      return notifyWithTone('info', title, message, timeoutMs);
    },
    success(title: string, message: string, timeoutMs?: number) {
      return notifyWithTone('success', title, message, timeoutMs);
    },
    warning(title: string, message: string, timeoutMs?: number) {
      return notifyWithTone('warning', title, message, timeoutMs);
    },
    error(title: string, message: string, timeoutMs?: number) {
      return notifyWithTone('error', title, message, timeoutMs);
    },
    dismiss,
    completeDismiss,
    clear
  };
}
