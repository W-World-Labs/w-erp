import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./i18n"; // G-11: i18n initialization (must be before components)
import "./index.css";
import { AuthProvider } from "./contexts/AuthContext";
import { reportErrorFromException } from "./lib/errorReporter";
import { initGlobalErrorHandlers } from "./lib/telemetry";

// Additive @w/xradar instrumentation (mirrors crashes to Radar). Runs
// alongside the existing local errorReporter handlers below — nothing replaced.
initGlobalErrorHandlers();

// Global error handlers — catch errors outside React ErrorBoundary scope
window.addEventListener('error', (event) => {
  console.error('[Global Error]', event.error?.message || event.message);
  console.error('[Stack]', event.error?.stack || 'No stack trace');
  reportErrorFromException(event.error || event.message, 'unhandled_exception');
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('[Unhandled Promise Rejection]', event.reason);
  if (event.reason?.stack) {
    console.error('[Stack]', event.reason.stack);
  }
  reportErrorFromException(event.reason, 'promise_rejection');
});

createRoot(document.getElementById("root")!).render(
  <AuthProvider>
    <App />
  </AuthProvider>
);
