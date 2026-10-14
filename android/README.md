# Apelles على Android

تم تجهيز Apelles للعمل على Android عبر نسخة WebAssembly قابلة للتثبيت كتطبيق **PWA**. هذا المسار يعيد استخدام محرّك Rust وواجهة `egui` الموجودة، بدل إعادة كتابة المحرّر في Kotlin.

## بناء النسخة

من جذر المستودع:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cd apps/apelles-web
trunk build --release
```

ستجد الملفات في `dist/web/`.

## تثبيتها على هاتف Android

1. ارفع محتويات `dist/web/` إلى خادم HTTPS.
2. افتح الرابط في Chrome على Android.
3. اختر **Install app** أو **Add to Home screen**.
4. سيعمل Apelles بواجهة مستقلة، ويمكنه استخدام التخزين المؤقت بعد أول تشغيل.

## ملاحظات الإصدار الأول

- يحتاج أول تشغيل إلى اتصال لتنزيل ملف WebAssembly؛ بعد ذلك يمكن لـ Service Worker تخزين ملفات الواجهة.
- فتح الملفات وحفظها يتمان عبر منتقي الملفات والتنزيلات في المتصفح.
- دعم اللمس والقلم يعتمد على دعم المتصفح و`egui`، وقد تحتاج بعض الأدوات إلى تحسين خاص بالشاشات الصغيرة.
- هذه نسخة Android مبنية على الويب وليست APK أصلية بعد. يمكن تغليف مجلد `dist/web/` لاحقًا عبر Capacitor أو Trusted Web Activity دون تغيير محرّك Apelles.

## التحقق

```bash
cargo test -p apelles-web
```
