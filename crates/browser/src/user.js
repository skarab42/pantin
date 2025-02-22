// Make sure Shield doesn't hit the network.
user_pref("app.normandy.api_url", "");

// Disable Firefox old build background check
user_pref("app.update.checkInstallTime", false);

// Disable automatically upgrading Firefox
//
// Note: Possible update tests could reset or flip the value to allow
// updates to be downloaded and applied.
user_pref("app.update.disabledForTesting", true);

// Enable the dump function, which sends messages to the system
// console
user_pref("browser.dom.window.dump.enabled", true);
user_pref("devtools.console.stdout.chrome", true);

// Do not restore the last open set of tabs if the browser crashed
user_pref("browser.sessionstore.resume_from_crash", false);

// Skip check for default browser on startup
user_pref("browser.shell.checkDefaultBrowser", false);

// Do not redirect user when a milestone upgrade of Firefox
// is detected
user_pref("browser.startup.homepage_override.mstone", "ignore");

// Start with a blank page (about:blank)
user_pref("browser.startup.page", 0);

// Disable the UI tour
user_pref("browser.uitour.enabled", false);

// Do not warn on quitting Firefox
user_pref("browser.warnOnQuit", false);

// Defensively disable data reporting systems
user_pref("datareporting.healthreport.documentServerURI", "http://%(server)s/dummy/healthreport/");
user_pref("datareporting.healthreport.logging.consoleEnabled", false);
user_pref("datareporting.healthreport.service.enabled", false);
user_pref("datareporting.healthreport.service.firstRun", false);
user_pref("datareporting.healthreport.uploadEnabled", false);

// Do not show datareporting policy notifications which can
// interfere with tests
user_pref("datareporting.policy.dataSubmissionEnabled", false);
user_pref("datareporting.policy.dataSubmissionPolicyBypassNotification", true);

// Disable the ProcessHangMonitor
user_pref("dom.ipc.reportProcessHangs", false);

// Only load extensions from the application and user profile
// AddonManager.SCOPE_PROFILE + AddonManager.SCOPE_APPLICATION
user_pref("extensions.autoDisableScopes", 0);
user_pref("extensions.enabledScopes", 5);

// Disable installing any distribution extensions or add-ons
user_pref("extensions.installDistroAddons", false);

// Turn off extension updates so they do not bother tests
user_pref("extensions.update.enabled", false);
user_pref("extensions.update.notifyUser", false);

// Allow the application to have focus even it runs in the
// background
user_pref("focusmanager.testmode", true);

// Disable useragent updates
user_pref("general.useragent.updates.enabled", false);

// Always use network provider for geolocation tests, so we bypass
// the macOS dialog raised by the corelocation provider
user_pref("geo.provider.testing", true);

// Do not scan wi-fi
user_pref("geo.wifi.scan", false);

// No hang monitor
user_pref("hangmonitor.timeout", 0);

// Disable idle-daily notifications to avoid expensive operations
// that may cause unexpected test timeouts.
user_pref("idle.lastDailyNotification", -1);

// Disable download and usage of OpenH264, and Widevine plugins
user_pref("media.gmp-manager.updateEnabled", false);

// Disable the GFX sanity window
user_pref("media.sanity-test.disabled", true);

// Do not automatically switch between offline and online
user_pref("network.manage-offline-status", false);

// Make sure SNTP requests do not hit the network
user_pref("network.sntp.pools", "%(server)s");

// Don't do network connections for mitm priming
user_pref("security.certerrors.mitm.priming.enabled", false);

// Ensure remote settings do not hit the network
user_pref("services.settings.server", "data:,#remote-settings-dummy/v1");

// Disable first run pages
user_pref("startup.homepage_welcome_url", "about:blank");
user_pref("startup.homepage_welcome_url.additional", "");

// asrouter expects a plain object or null
user_pref("browser.newtabpage.activity-stream.asrouter.providers.cfr", "null");
// TODO: Remove once minimum supported Firefox release is 93.
user_pref("browser.newtabpage.activity-stream.asrouter.providers.cfr-fxa", "null");

// TODO: Remove once minimum supported Firefox release is 128.
user_pref("browser.newtabpage.activity-stream.asrouter.providers.snippets", "null");

user_pref("browser.newtabpage.activity-stream.asrouter.providers.message-groups", "null");
// TODO: Remove once minimum supported Firefox release is 126.
user_pref("browser.newtabpage.activity-stream.asrouter.providers.whats-new-panel", "null");
user_pref("browser.newtabpage.activity-stream.asrouter.providers.messaging-experiments", "null");
user_pref("browser.newtabpage.activity-stream.feeds.system.topstories", false);

// TODO: Remove once minimum supported Firefox release is 128.
user_pref("browser.newtabpage.activity-stream.feeds.snippets", false);

user_pref("browser.newtabpage.activity-stream.tippyTop.service.endpoint", "");
user_pref("browser.newtabpage.activity-stream.discoverystream.config", "[]");

// For Activity Stream firstrun page, use an empty string to avoid fetching.
user_pref("browser.newtabpage.activity-stream.fxaccounts.endpoint", "");

// Prevent starting into safe mode after application crashes
user_pref("toolkit.startup.max_resumed_crashes", -1);

// Disable webapp updates.
user_pref("browser.webapps.checkForUpdates", 0);

// THESE LINES WERE AUTOMATICALLY ADDED BY PANTIN DURING COMPILATION
