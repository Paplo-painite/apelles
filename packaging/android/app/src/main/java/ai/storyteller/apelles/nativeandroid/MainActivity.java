package ai.storyteller.apelles.nativeandroid;

import android.app.NativeActivity;
import android.content.ClipDescription;
import android.content.ClipboardManager;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.Build;
import android.provider.OpenableColumns;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.util.Log;
import android.widget.Toast;
import android.app.AlertDialog;
import android.graphics.Typeface;
import android.text.TextUtils;
import android.view.Gravity;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

import androidx.core.graphics.Insets;
import androidx.core.content.FileProvider;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.Locale;
import org.json.JSONArray;
import org.json.JSONObject;
import java.util.HashMap;
import java.util.Map;

/** NativeActivity shell for Apelles's Rust/eframe renderer. */
public final class MainActivity extends NativeActivity {
    static {
        System.loadLibrary("apelles_android");
    }

    private static final int FILE_PICKER_REQUEST = 9017;
    private static final long MAX_IMPORT_BYTES = 512L * 1024L * 1024L;
    private static final long MAX_FONT_BYTES = 64L * 1024L * 1024L;
    private static final long MAX_CLIPBOARD_PIXELS = 100_000_000L;
    private static final String IMPORT_PREFIX = "apelles-import-";
    private static final String CLIPBOARD_PREFIX = "apelles-clipboard-";
    private static final String SAVE_PREFIX = "apelles-save-";
    private static final String UPDATE_PREFS = "apelles-updates";
    private static final String UPDATE_CHECKED_AT = "checked_at";
    private static final long UPDATE_CHECK_INTERVAL_MS = 24L * 60L * 60L * 1000L;
    private static final String RELEASES_API = "https://api.github.com/repos/Paplo-painite/apelles/releases?per_page=10";
    private static final String RELEASES_PAGE = "https://github.com/Paplo-painite/apelles/releases";
    private final Handler handler = new Handler(Looper.getMainLooper());
    private boolean pickerOpen = false;
    private boolean savePickerOpen = false;
    private boolean fontPickerOpen = false;
    private final Map<String, Uri> pendingSaveUris = new HashMap<>();
    private String lastClipboardKey = "";
    private boolean updateCheckStarted = false;
    private final Runnable barStatePoller = new Runnable() {
        @Override public void run() {
            applySystemBars();
            maybeLaunchFilePicker();
            commitCompletedSaves();
            syncClipboard();
            handler.postDelayed(this, 1000L);
        }
    };

    private static native void nativeInsetsChanged(int left, int top, int right, int bottom);
    private static native boolean nativeShouldHideSystemBars();
    private static native void nativeRequestAutosave();
    private static native int nativeTakeFilePickerRequest();
    private static native String nativeTakeSaveSuggestedName();
    private static native void nativeFontPickerResult(String path, String name);
    private static native void nativeFontPickerCancelled();
    private static native void nativeFilePickerOpenResult(String path, String name);
    private static native void nativeFilePickerSaveResult(String tempPath, String uri);
    private static native void nativeFilePickerCancelled();
    private static native String nativeTakeSaveCompletion();
    private static native byte[] nativeTakeClipboardSet();
    private static native void nativeClipboardImageResult(byte[] pixels, int width, int height);
    private static native boolean nativeTakeTextClipboardRequest();
    private static native void nativeTextClipboardResult(String text);

    @Override protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        configureWindow();
        installInsetsListener();
        cleanupTemporaryFiles();
        applySystemBars();
        handler.post(barStatePoller);
        handler.postDelayed(this::checkForUpdatesIfDue, 1500L);
    }

    @Override protected void onResume() {
        super.onResume();
        handler.removeCallbacks(barStatePoller);
        handler.post(barStatePoller);
        if (!updateCheckStarted) handler.postDelayed(this::checkForUpdatesIfDue, 1000L);
    }

    @Override protected void onPause() {
        try {
            nativeRequestAutosave();
        } catch (UnsatisfiedLinkError ignored) {
            // Native code may still be shutting down during process teardown.
        }
        handler.removeCallbacks(barStatePoller);
        super.onPause();
    }

    @Override public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            configureWindow();
            applySystemBars();
            ViewCompat.requestApplyInsets(getWindow().getDecorView());
        }
    }

    @Override protected void onDestroy() {
        handler.removeCallbacks(barStatePoller);
        super.onDestroy();
    }

    private void configureWindow() {
        Window window = getWindow();
        WindowCompat.setDecorFitsSystemWindows(window, false);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            WindowManager.LayoutParams attributes = window.getAttributes();
            attributes.layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            window.setAttributes(attributes);
        }
    }

    /**
     * Check for a newer GitHub Release at most once per day. The check never downloads or installs
     * an APK: the user must press "تحديث الآن", which opens the release page/download in a browser.
     */
    private void checkForUpdatesIfDue() {
        if (updateCheckStarted || isFinishing() || (Build.VERSION.SDK_INT >= 17 && isDestroyed())) return;
        updateCheckStarted = true;
        android.content.SharedPreferences preferences = getSharedPreferences(UPDATE_PREFS, MODE_PRIVATE);
        long lastCheck = preferences.getLong(UPDATE_CHECKED_AT, 0L);
        long now = System.currentTimeMillis();
        if (now - lastCheck < UPDATE_CHECK_INTERVAL_MS) return;
        preferences.edit().putLong(UPDATE_CHECKED_AT, now).apply();

        new Thread(() -> {
            try {
                HttpURLConnection connection = (HttpURLConnection) new URL(RELEASES_API).openConnection();
                connection.setConnectTimeout(5000);
                connection.setReadTimeout(7000);
                connection.setRequestProperty("Accept", "application/vnd.github+json");
                connection.setRequestProperty("User-Agent", "Apelles-Android-Update-Checker");
                String body;
                try (InputStream input = connection.getInputStream(); java.io.ByteArrayOutputStream output = new java.io.ByteArrayOutputStream()) {
                    byte[] buffer = new byte[8192];
                    int count;
                    int total = 0;
                    while ((count = input.read(buffer)) != -1 && total < 2 * 1024 * 1024) {
                        output.write(buffer, 0, count);
                        total += count;
                    }
                    body = output.toString(StandardCharsets.UTF_8.name());
                } finally {
                    connection.disconnect();
                }
                JSONObject release = newestRelease(new JSONArray(body));
                if (release == null) return;
                String remoteTag = release.optString("tag_name", "");
                if (!isNewerVersion(remoteTag, currentVersionName())) return;
                String notes = release.optString("body", "").trim();
                String pageUrl = release.optString("html_url", RELEASES_PAGE);
                String apkUrl = apkAssetUrl(release);
                handler.post(() -> showUpdateDialog(remoteTag, notes, pageUrl, apkUrl));
            } catch (Exception ignored) {
                // Update checks are best-effort and must never affect offline editing.
            }
        }, "apelles-update-check").start();
    }

    private JSONObject newestRelease(JSONArray releases) {
        JSONObject best = null;
        for (int i = 0; i < releases.length(); i++) {
            JSONObject candidate = releases.optJSONObject(i);
            if (candidate == null || candidate.optBoolean("draft", false)) continue;
            String tag = candidate.optString("tag_name", "");
            if (!tag.startsWith("v") || !isNewerVersion(tag, best == null ? "0.0.0" : best.optString("tag_name", "0.0.0"))) continue;
            best = candidate;
        }
        return best;
    }

    private String apkAssetUrl(JSONObject release) {
        JSONArray assets = release.optJSONArray("assets");
        if (assets != null) {
            for (int i = 0; i < assets.length(); i++) {
                JSONObject asset = assets.optJSONObject(i);
                if (asset != null && asset.optString("name", "").toLowerCase(Locale.ROOT).endsWith(".apk")) {
                    return asset.optString("browser_download_url", "");
                }
            }
        }
        return "";
    }

    private String currentVersionName() {
        try {
            return getPackageManager().getPackageInfo(getPackageName(), 0).versionName;
        } catch (Exception ignored) {
            return "0.0.0";
        }
    }

    private boolean isNewerVersion(String remote, String local) {
        int[] r = versionParts(remote);
        int[] l = versionParts(local);
        for (int i = 0; i < 3; i++) {
            if (r[i] != l[i]) return r[i] > l[i];
        }
        return false;
    }

    private int[] versionParts(String value) {
        int[] result = new int[]{0, 0, 0};
        if (value == null) return result;
        String clean = value.replaceFirst("^[vV]", "").split("[-+]")[0];
        String[] parts = clean.split("\\.");
        for (int i = 0; i < Math.min(parts.length, 3); i++) {
            try { result[i] = Integer.parseInt(parts[i]); } catch (NumberFormatException ignored) { return new int[]{0, 0, 0}; }
        }
        return result;
    }

    private void showUpdateDialog(String tag, String notes, String pageUrl, String apkUrl) {
        if (isFinishing() || (Build.VERSION.SDK_INT >= 17 && isDestroyed())) return;
        LinearLayout content = new LinearLayout(this);
        content.setOrientation(LinearLayout.VERTICAL);
        content.setPadding(48, 8, 48, 0);

        TextView summary = new TextView(this);
        summary.setText("يتوفر إصدار جديد من Apelles\n\nالإصدار الجديد: " + tag + "\n\nإصلاحات وتحسينات:");
        summary.setTextSize(17f);
        summary.setTypeface(Typeface.DEFAULT, Typeface.BOLD);
        summary.setGravity(Gravity.RIGHT);
        content.addView(summary);

        TextView details = new TextView(this);
        details.setText(TextUtils.isEmpty(notes) ? "راجع صفحة الإصدار لمعرفة التفاصيل." : notes);
        details.setTextSize(15f);
        details.setGravity(Gravity.RIGHT);
        details.setPadding(0, 18, 0, 12);
        ScrollView scroll = new ScrollView(this);
        scroll.addView(details);
        content.addView(scroll, new LinearLayout.LayoutParams(-1, 360));

        AlertDialog dialog = new AlertDialog.Builder(this)
                .setTitle("تحديث متاح")
                .setView(content)
                .setNegativeButton("لاحقًا", null)
                .setPositiveButton("تحديث الآن", null)
                .create();
        dialog.setOnShowListener(ignored -> dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener(view -> {
            // Open the release page instead of sending a remote APK directly to an installer.
            // Some OxygenOS installers receive a content:// URI without a read grant and report
            // "has no access to content://media/external/downloads/...". The page flow lets the
            // browser/download manager save the APK, after which Android Files can install it.
            try {
                Intent intent = new Intent(Intent.ACTION_VIEW, Uri.parse(pageUrl));
                intent.addCategory(Intent.CATEGORY_BROWSABLE);
                startActivity(intent);
            } catch (Exception error) {
                try { startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(RELEASES_PAGE))); } catch (Exception ignoredAgain) { }
            }
            dialog.dismiss();
        }));
        dialog.show();
    }

    private void installInsetsListener() {
        View decor = getWindow().getDecorView();
        ViewCompat.setOnApplyWindowInsetsListener(decor, (view, insets) -> {
            Insets safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() | WindowInsetsCompat.Type.displayCutout());
            try {
                nativeInsetsChanged(safe.left, safe.top, safe.right, safe.bottom);
            } catch (UnsatisfiedLinkError ignored) {
                // NativeActivity may still be completing its library handoff.
            }
            return insets;
        });
        ViewCompat.requestApplyInsets(decor);
    }

    private void applySystemBars() {
        View decor = getWindow().getDecorView();
        WindowInsetsControllerCompat controller = WindowCompat.getInsetsController(getWindow(), decor);
        if (controller == null) return;
        controller.setSystemBarsBehavior(WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
        boolean hide = true;
        try {
            hide = nativeShouldHideSystemBars();
        } catch (UnsatisfiedLinkError ignored) {
            // Keep immersive mode during native startup.
        }
        if (hide) {
            controller.hide(WindowInsetsCompat.Type.statusBars() | WindowInsetsCompat.Type.navigationBars());
        } else {
            controller.show(WindowInsetsCompat.Type.statusBars() | WindowInsetsCompat.Type.navigationBars());
        }
        ViewCompat.requestApplyInsets(decor);
    }

    private void maybeLaunchFilePicker() {
        if (pickerOpen) return;
        final int request;
        try {
            request = nativeTakeFilePickerRequest();
        } catch (UnsatisfiedLinkError ignored) {
            return;
        }
        if (request == 0) return;
        Intent intent;
        if (request == 2) {
            savePickerOpen = true;
            String suggested = nativeTakeSaveSuggestedName();
            if (suggested == null || suggested.trim().isEmpty()) suggested = "Apelles.apelles";
            intent = new Intent(Intent.ACTION_CREATE_DOCUMENT);
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            intent.setType(mimeForName(suggested));
            intent.putExtra(Intent.EXTRA_TITLE, suggested);
        } else if (request == 3) {
            savePickerOpen = false;
            fontPickerOpen = true;
            intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            intent.setType("*/*");
            intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[]{"font/ttf", "font/otf", "font/ttc", "application/x-font-ttf", "application/x-font-opentype"});
            intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);
        } else {
            savePickerOpen = false;
            fontPickerOpen = false;
            intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            intent.setType("*/*");
            intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, false);
        }
        pickerOpen = true;
        startActivityForResult(intent, FILE_PICKER_REQUEST);
    }

    @Override protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != FILE_PICKER_REQUEST) return;
        pickerOpen = false;
        if (resultCode != RESULT_OK || data == null || (data.getData() == null && data.getClipData() == null)) {
            if (fontPickerOpen) {
                fontPickerOpen = false;
                nativeFontPickerCancelled();
                return;
            }
            savePickerOpen = false;
            nativeFilePickerCancelled();
            return;
        }
        Uri uri = data.getData();
        try {
            if (fontPickerOpen) {
                fontPickerOpen = false;
                if (data.getClipData() != null) {
                    for (int i = 0; i < data.getClipData().getItemCount(); i++) copyFont(data.getClipData().getItemAt(i).getUri());
                } else {
                    copyFont(uri);
                }
                nativeFontPickerCancelled();
            } else if (savePickerOpen) {
                savePickerOpen = false;
                String selectedName = displayName(uri);
                String suffix = extensionOf(selectedName);
                if (suffix.isEmpty()) suffix = ".apelles";
                File temp = new File(getCacheDir(), "apelles-save-" + System.nanoTime() + suffix);
                if (!temp.createNewFile()) throw new IllegalStateException("Unable to create temporary save file");
                pendingSaveUris.put(temp.getAbsolutePath(), uri);
                nativeFilePickerSaveResult(temp.getAbsolutePath(), uri.toString());
            } else {
                String name = displayName(uri);
                File out = new File(getCacheDir(), IMPORT_PREFIX + System.nanoTime() + "-" + safeName(name));
                try (InputStream input = getContentResolver().openInputStream(uri);
                     FileOutputStream output = new FileOutputStream(out)) {
                    if (input == null) throw new IllegalStateException("Unable to open selected file");
                    byte[] buffer = new byte[1024 * 1024];
                    int count;
                    long total = 0L;
                    while ((count = input.read(buffer)) != -1) {
                        total += count;
                        if (total > MAX_IMPORT_BYTES) throw new IllegalStateException("Selected file is larger than 512 MB");
                        output.write(buffer, 0, count);
                    }
                }
                nativeFilePickerOpenResult(out.getAbsolutePath(), name);
            }
        } catch (Exception error) {
            if (!savePickerOpen && !fontPickerOpen && uri != null) {
                // The import file is disposable and must not survive a failed import.
                cleanupTemporaryFiles();
            }
            showError("File picker failed: " + error.getMessage(), error);
            nativeFilePickerCancelled();
        }
    }

    private void commitCompletedSaves() {
        String tempPath;
        try {
            tempPath = nativeTakeSaveCompletion();
        } catch (UnsatisfiedLinkError ignored) {
            return;
        }
        if (tempPath == null || tempPath.isEmpty()) return;
        Uri uri = pendingSaveUris.remove(tempPath);
        if (uri == null) return;
        File temp = new File(tempPath);
        long expected = temp.length();
        if (expected <= 0L) {
            temp.delete();
            deleteDestination(uri);
            showError("Save failed: the temporary file is empty", null);
            return;
        }
        try (InputStream input = new java.io.FileInputStream(temp);
             OutputStream output = getContentResolver().openOutputStream(uri, "wt")) {
            if (output == null) throw new IllegalStateException("Unable to open selected destination");
            byte[] buffer = new byte[1024 * 1024];
            int count;
            long copied = 0L;
            while ((count = input.read(buffer)) != -1) {
                output.write(buffer, 0, count);
                copied += count;
            }
            output.flush();
            if (copied != expected || copied == 0L) throw new IllegalStateException("destination size mismatch: expected " + expected + ", wrote " + copied);
            temp.delete();
        } catch (Exception error) {
            deleteDestination(uri);
            showError("Save failed: " + error.getMessage(), error);
            temp.delete();
        }
    }

    private void syncClipboard() {
        if (nativeTakeTextClipboardRequest()) {
            ClipboardManager textClipboard = (ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
            if (textClipboard != null && textClipboard.hasPrimaryClip()) {
                android.content.ClipData textClip = textClipboard.getPrimaryClip();
                if (textClip != null && textClip.getItemCount() > 0) {
                    CharSequence text = textClip.getItemAt(0).coerceToText(this);
                    if (text != null) nativeTextClipboardResult(text.toString());
                }
            }
        }
        byte[] pending = nativeTakeClipboardSet();
        if (pending != null && pending.length >= 8) {
            int width = readIntLE(pending, 0);
            int height = readIntLE(pending, 4);
            long expectedLong = (long) width * height * 4L;
            if (width > 0 && height > 0 && (long) width * height <= MAX_CLIPBOARD_PIXELS && expectedLong <= Integer.MAX_VALUE && pending.length == expectedLong + 8L) {
                int[] pixels = new int[width * height];
                for (int i = 0; i < pixels.length; i++) {
                    int p = 8 + i * 4;
                    pixels[i] = ((pending[p + 3] & 0xff) << 24) | ((pending[p] & 0xff) << 16) | ((pending[p + 1] & 0xff) << 8) | (pending[p + 2] & 0xff);
                }
                Bitmap bitmap = Bitmap.createBitmap(pixels, width, height, Bitmap.Config.ARGB_8888);
                    Uri uri = cacheBitmap(bitmap);
                    if (!Uri.EMPTY.equals(uri)) {
                        ClipboardManager clipboard = (ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
                        clipboard.setPrimaryClip(android.content.ClipData.newUri(getContentResolver(), "Apelles", uri));
                    }
            }
        }

        ClipboardManager clipboard = (ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
        if (clipboard == null || !clipboard.hasPrimaryClip()) return;
        android.content.ClipData clip = clipboard.getPrimaryClip();
        if (clip == null || clip.getItemCount() == 0 || !clip.getDescription().hasMimeType("image/*")) return;
        Uri uri = clip.getItemAt(0).getUri();
        if (uri == null) return;
        String key = uri.toString();
        if (key.equals(lastClipboardKey)) return;
        lastClipboardKey = key;
        try (InputStream input = getContentResolver().openInputStream(uri)) {
            if (input == null) return;
            Bitmap decoded = BitmapFactory.decodeStream(input);
            if (decoded == null) return;
            Bitmap bitmap = decoded.getConfig() == Bitmap.Config.ARGB_8888 ? decoded : decoded.copy(Bitmap.Config.ARGB_8888, false);
            if (bitmap == null) return;
            int width = bitmap.getWidth();
            int height = bitmap.getHeight();
            if ((long) width * height > MAX_CLIPBOARD_PIXELS) return;
            int[] pixels = new int[width * height];
            bitmap.getPixels(pixels, 0, width, 0, 0, width, height);
            byte[] rgba = new byte[pixels.length * 4];
            for (int i = 0; i < pixels.length; i++) {
                int color = pixels[i];
                rgba[i * 4] = (byte) ((color >> 16) & 0xff);
                rgba[i * 4 + 1] = (byte) ((color >> 8) & 0xff);
                rgba[i * 4 + 2] = (byte) (color & 0xff);
                rgba[i * 4 + 3] = (byte) ((color >> 24) & 0xff);
            }
            nativeClipboardImageResult(rgba, width, height);
        } catch (Exception ignored) { }
    }

    private Uri cacheBitmap(Bitmap bitmap) {
        File dir = new File(getCacheDir(), "apelles-clipboard");
        if (!dir.exists() && !dir.mkdirs()) return Uri.EMPTY;
        deleteFilesWithPrefix(dir, CLIPBOARD_PREFIX);
        File file = new File(dir, CLIPBOARD_PREFIX + "current.png");
        try (FileOutputStream output = new FileOutputStream(file)) {
            bitmap.compress(Bitmap.CompressFormat.PNG, 100, output);
            return FileProvider.getUriForFile(this, getPackageName() + ".fileprovider", file);
        } catch (Exception error) {
            file.delete();
            return Uri.EMPTY;
        }
    }

    private void cleanupTemporaryFiles() {
        deleteFilesWithPrefix(getCacheDir(), IMPORT_PREFIX);
        deleteFilesWithPrefix(getCacheDir(), SAVE_PREFIX);
        File clipboardDir = new File(getCacheDir(), "apelles-clipboard");
        deleteFilesWithPrefix(clipboardDir, CLIPBOARD_PREFIX);
    }

    private void deleteFilesWithPrefix(File dir, String prefix) {
        if (dir == null || !dir.isDirectory()) return;
        File[] files = dir.listFiles();
        if (files == null) return;
        for (File file : files) {
            if (file.isFile() && file.getName().startsWith(prefix)) file.delete();
        }
    }

    private int readIntLE(byte[] bytes, int offset) {
        return (bytes[offset] & 0xff) | ((bytes[offset + 1] & 0xff) << 8) | ((bytes[offset + 2] & 0xff) << 16) | ((bytes[offset + 3] & 0xff) << 24);
    }

    private String displayName(Uri uri) {
        String name = null;
        try (Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null)) {
            if (cursor != null && cursor.moveToFirst()) name = cursor.getString(0);
        } catch (Exception ignored) { }
        if (name == null || name.trim().isEmpty()) name = "image";
        return name;
    }

    private String safeName(String name) {
        return name.replaceAll("[^a-zA-Z0-9._-]", "_");
    }
    private void copyFont(Uri uri) throws Exception {
        if (uri == null) return;
        String name = displayName(uri);
        String ext = extensionOf(name);
        if (!(ext.equals(".ttf") || ext.equals(".otf") || ext.equals(".ttc"))) throw new IllegalStateException("Unsupported font file: " + name);
        File dir = new File(getFilesDir(), "Apelles/fonts");
        if (!dir.exists() && !dir.mkdirs()) throw new IllegalStateException("Unable to create font directory");
        File out = new File(dir, safeName(name));
        try (InputStream input = getContentResolver().openInputStream(uri); FileOutputStream output = new FileOutputStream(out)) {
            if (input == null) throw new IllegalStateException("Unable to open font file");
            byte[] buffer = new byte[64 * 1024];
            int count;
            long total = 0L;
            while ((count = input.read(buffer)) != -1) {
                total += count;
                if (total > MAX_FONT_BYTES) throw new IllegalStateException("Font is larger than 64 MB");
                output.write(buffer, 0, count);
            }
        }
        nativeFontPickerResult(out.getAbsolutePath(), name);
    }
    private String extensionOf(String name) {
        int dot = name.lastIndexOf('.');
        return dot >= 0 ? name.substring(dot).toLowerCase(java.util.Locale.ROOT).replaceAll("[^a-z0-9.]", "") : "";
    }
    private String mimeForName(String name) {
        switch (extensionOf(name)) {
            case ".png": return "image/png";
            case ".jpg": case ".jpeg": return "image/jpeg";
            case ".webp": return "image/webp";
            case ".tif": case ".tiff": return "image/tiff";
            case ".psd": return "image/vnd.adobe.photoshop";
            case ".pcraft": return "application/octet-stream";
            default: return "application/octet-stream";
        }
    }
    private void deleteDestination(Uri uri) {
        try { getContentResolver().delete(uri, null, null); } catch (Exception error) { Log.e("Apelles", "Unable to delete failed destination", error); }
    }
    private void showError(String message, Exception error) {
        Log.e("Apelles", message, error);
        Toast.makeText(this, message, Toast.LENGTH_LONG).show();
    }
}
