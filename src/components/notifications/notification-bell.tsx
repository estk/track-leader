"use client";

import { useState, useEffect, useRef } from "react";
import Link from "next/link";
import { api, Notification } from "@/lib/api";
import { formatDistanceToNow } from "@/lib/utils";

function getNotificationMessage(notification: Notification): string {
  switch (notification.notification_type) {
    case "follow":
      return `${notification.actor_name || "Someone"} started following you`;
    case "kudos":
      return `${notification.actor_name || "Someone"} gave kudos to your activity`;
    case "comment":
      return `${notification.actor_name || "Someone"} commented on your activity`;
    case "crown_achieved":
      return notification.message || "You achieved a new crown!";
    case "crown_lost":
      return notification.message || "You lost a crown";
    case "pr":
      return notification.message || "You set a new personal record!";
    default:
      return notification.message || "New notification";
  }
}

function getNotificationLink(notification: Notification): string | null {
  switch (notification.notification_type) {
    case "follow":
      return notification.actor_id ? `/profile/${notification.actor_id}` : null;
    case "kudos":
    case "comment":
      return notification.target_id ? `/activities/${notification.target_id}` : null;
    case "crown_achieved":
    case "crown_lost":
      return notification.target_id ? `/segments/${notification.target_id}` : null;
    case "pr":
      return notification.target_id ? `/activities/${notification.target_id}` : null;
    default:
      return null;
  }
}

export function NotificationBell() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Fetch notifications on mount and periodically
  useEffect(() => {
    fetchNotifications();
    const interval = setInterval(fetchNotifications, 60000); // Poll every minute
    return () => clearInterval(interval);
  }, []);

  const fetchNotifications = async () => {
    try {
      const response = await api.getNotifications(10, 0);
      setNotifications(response.notifications);
      setUnreadCount(response.unread_count);
    } catch {
      // Error fetching notifications
    }
  };

  const handleOpen = async () => {
    setIsOpen(!isOpen);
    if (!isOpen) {
      setLoading(true);
      await fetchNotifications();
      setLoading(false);
    }
  };

  const handleMarkRead = async (notificationId: string) => {
    try {
      await api.markNotificationRead(notificationId);
      setNotifications((prev) =>
        prev.map((n) =>
          n.id === notificationId ? { ...n, read_at: new Date().toISOString() } : n
        )
      );
      setUnreadCount((c) => Math.max(0, c - 1));
    } catch {
      // Error marking as read
    }
  };

  const handleMarkAllRead = async () => {
    try {
      await api.markAllNotificationsRead();
      setNotifications((prev) =>
        prev.map((n) => ({ ...n, read_at: n.read_at || new Date().toISOString() }))
      );
      setUnreadCount(0);
    } catch {
      // Error marking all as read
    }
  };

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={handleOpen}
        className="relative p-2 text-muted-foreground hover:text-foreground transition-colors"
        aria-label="Notifications"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          className="w-6 h-6"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0"
          />
        </svg>
        {unreadCount > 0 && (
          <span className="absolute top-0 right-0 inline-flex items-center justify-center w-5 h-5 text-xs font-bold text-white bg-red-500 rounded-full">
            {unreadCount > 9 ? "9+" : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-80 bg-background border rounded-lg shadow-lg z-50">
          <div className="flex items-center justify-between p-3 border-b">
            <h3 className="font-semibold">Notifications</h3>
            {unreadCount > 0 && (
              <button
                onClick={handleMarkAllRead}
                className="text-xs text-primary hover:underline"
              >
                Mark all read
              </button>
            )}
          </div>

          <div className="max-h-96 overflow-y-auto">
            {loading ? (
              <div className="p-4 text-center text-muted-foreground text-sm">
                Loading...
              </div>
            ) : notifications.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground text-sm">
                No notifications yet
              </div>
            ) : (
              <ul>
                {notifications.map((notification) => {
                  const link = getNotificationLink(notification);
                  const isUnread = !notification.read_at;
                  const content = (
                    <div
                      className={`p-3 border-b last:border-b-0 hover:bg-muted/50 transition-colors ${
                        isUnread ? "bg-primary/5" : ""
                      }`}
                    >
                      <p className="text-sm">{getNotificationMessage(notification)}</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        {formatDistanceToNow(new Date(notification.created_at))}
                      </p>
                    </div>
                  );

                  return (
                    <li key={notification.id}>
                      {link ? (
                        <Link
                          href={link}
                          onClick={() => {
                            if (isUnread) handleMarkRead(notification.id);
                            setIsOpen(false);
                          }}
                        >
                          {content}
                        </Link>
                      ) : (
                        <button
                          onClick={() => {
                            if (isUnread) handleMarkRead(notification.id);
                          }}
                          className="w-full text-left"
                        >
                          {content}
                        </button>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}
          </div>

          <div className="p-2 border-t">
            <Link
              href="/notifications"
              onClick={() => setIsOpen(false)}
              className="block text-center text-sm text-primary hover:underline"
            >
              View all notifications
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
