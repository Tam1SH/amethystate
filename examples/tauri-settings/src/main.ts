import {AppSettings} from "./bindings/rpstate";

async function initApp() {
  // 1. Batch-load the entire settings slice in a single round-trip IPC call on startup.
  const settings = await AppSettings.load();

  const usernameInput = document.querySelector("#username-input") as HTMLInputElement | null;
  const usernameDisplay = document.querySelector("#username-display");
  const counterBtn = document.querySelector("#counter-btn");
  const counterDisplay = document.querySelector("#counter-display");
  const themeBtn = document.querySelector("#theme-btn");
  const themeDisplay = document.querySelector("#theme-display");

  // 2. Synchronous Username Sync (Optimistic update with explicit background flush)
  if (usernameInput && usernameDisplay) {
    // Read the cached state synchronously from memory (no await required)
    usernameInput.value = settings.username.value ?? "";
    usernameDisplay.textContent = settings.username.value;

    usernameInput.addEventListener("input", async () => {
      // Optimistic update: instantly sets local JS memory and triggers a background write to Rust
      settings.username.value = usernameInput.value;

      // Note on settings.save():
      // settings.save() forces an immediate physical flush of the buffered memory slice to disk.
      // Since background debounced/lazy persistence is handled automatically by the backend database,
      // manually awaiting save() here is optional. It is commented out here to allow zero-latency writes.
      //
      // try {
      //   await settings.save();
      // } catch (err) {
      //   console.error("Failed to commit username to disk:", err);
      // }
    });

    // Reactive subscription to smoothly sync values in real-time across components
    settings.username.subscribe((val) => {
      usernameDisplay.textContent = val;
      if (document.activeElement !== usernameInput) {
        usernameInput.value = val;
      }
    });
  }

  // 3. Persistent Counter
  if (counterBtn && counterDisplay) {
    counterDisplay.textContent = (settings.counter.value ?? 0).toString();

    settings.counter.subscribe((val) => {
      counterDisplay.textContent = val.toString();
    });

    counterBtn.addEventListener("click", async () => {
      const current = settings.counter.value ?? 0;
      settings.counter.value = current + 1;

      // try {
      //   await settings.save();
      // } catch (err) {
      //   console.error("Failed to commit counter to disk:", err);
      // }
    });
  }

  // 4. Synchronous Theme Toggle
  if (themeBtn && themeDisplay) {
    const applyTheme = (val: string | null) => {
      themeDisplay.textContent = val ?? "light";
      if (val === "dark") {
        document.body.style.backgroundColor = "#2f2f2f";
        document.body.style.color = "#f6f6f6";
      } else {
        document.body.style.backgroundColor = "#f6f6f6";
        document.body.style.color = "#0f0f0f";
      }
    };

    applyTheme(settings.theme.value);

    settings.theme.subscribe((val) => {
      applyTheme(val);
    });

    themeBtn.addEventListener("click", async () => {
      const current = settings.theme.value;
      settings.theme.value = current === "light" ? "dark" : "light";

      // try {
      //   await settings.save();
      // } catch (err) {
      //   console.error("Failed to commit theme to disk:", err);
      // }
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // Initialize the store-backed application slice loader
  initApp().catch(console.error);
});